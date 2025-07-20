use crate::clipboard_item::{ClipboardContentType, ClipboardItem};
use crate::monitor::ClipboardMonitor;
use crate::storage::Storage;
use base64::prelude::*;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

const MAX_HISTORY_SIZE: usize = 1000;
const MAX_CONTENT_SIZE: usize = 10_000_000; // 10MB limit for individual entries

#[derive(Debug)]
pub struct ClipboardManager {
    history: Arc<Mutex<VecDeque<ClipboardItem>>>,
    storage: Storage,
}

impl ClipboardManager {
    pub async fn new() -> io::Result<Self> {
        let storage = Storage::new()?;
        let history = Arc::new(Mutex::new(storage.load_history().await?));

        Ok(Self { history, storage })
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub async fn new_with_storage(storage: Storage) -> io::Result<Self> {
        let history = Arc::new(Mutex::new(storage.load_history().await?));
        Ok(Self { history, storage })
    }

    // Public method for testing - creates an empty manager
    #[allow(dead_code)] // Used by tests
    pub fn new_empty() -> Self {
        let history = Arc::new(Mutex::new(VecDeque::new()));
        // Create a dummy storage for testing
        let storage = Storage::new_with_file(std::path::PathBuf::from("test_history.json"))
            .unwrap_or_else(|_| {
                // Fallback to a simple path if that fails
                Storage::new_with_file(std::path::PathBuf::from("./test.json")).unwrap()
            });

        Self { history, storage }
    }

    pub async fn add_clipboard_item(&self, item: ClipboardItem) -> io::Result<()> {
        // Check content size limit
        let item_size = item.get_size_bytes();
        if item_size > MAX_CONTENT_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Content too large: {item_size} bytes (max: {MAX_CONTENT_SIZE} bytes)"),
            ));
        }

        let mut history = self.history.lock().await;

        // Skip duplicates by comparing content hash
        if let Some(last) = history.front() {
            if last.content_hash == item.content_hash {
                return Ok(());
            }
        }

        history.push_front(item);

        // Maintain max size
        if history.len() > MAX_HISTORY_SIZE {
            history.pop_back();
        }

        drop(history);
        self.save_history().await
    }

    pub async fn get_history(&self) -> Vec<ClipboardItem> {
        let history = self.history.lock().await;
        history.iter().cloned().collect()
    }

    pub async fn search_history(&self, query: &str) -> Vec<(usize, ClipboardItem)> {
        let history = self.history.lock().await;

        // Search across different content types using display_content (without type prefix)
        let matches: Vec<(usize, ClipboardItem)> = history
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                let content = item.display_content();
                content.to_lowercase().contains(&query.to_lowercase())
            })
            .map(|(idx, item)| (idx, item.clone()))
            .collect();

        matches
    }

    pub async fn fuzzy_search_history(&self, query: &str) -> Vec<(usize, ClipboardItem, i64)> {
        let history = self.history.lock().await;
        let matcher = SkimMatcherV2::default();

        let mut fuzzy_matches: Vec<(usize, ClipboardItem, i64)> = history
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let content = item.display_content();
                matcher
                    .fuzzy_match(&content, query)
                    .map(|score| (idx, item.clone(), score))
            })
            .collect();

        // Sort by fuzzy match score (higher is better)
        fuzzy_matches.sort_by(|a, b| b.2.cmp(&a.2));
        fuzzy_matches
    }

    pub async fn copy_item_to_clipboard(&self, index: usize) -> io::Result<bool> {
        let history = self.history.lock().await;
        if let Some(item) = history.get(index) {
            let item_clone = item.clone();
            drop(history);

            // Use blocking task for clipboard operation
            let result = tokio::task::spawn_blocking(move || {
                let mut clipboard =
                    arboard::Clipboard::new().map_err(|_| "Failed to access clipboard")?;

                match &item_clone.content {
                    ClipboardContentType::Text(text) => {
                        clipboard
                            .set_text(text.clone())
                            .map_err(|_| "Failed to set clipboard text")?;
                    }
                    ClipboardContentType::Image { data, width, height, .. } => {
                        // Decode base64 PNG data and convert back to RGBA for clipboard
                        if let Ok(png_data) = BASE64_STANDARD.decode(data) {
                            // Validate that we have valid dimensions
                            if *width > 0 && *height > 0 {
                                // Convert PNG back to RGBA format for arboard
                                match ClipboardMonitor::png_to_rgba(&png_data) {
                                    Ok(rgba_data) => {
                                        let img = arboard::ImageData {
                                            width: *width as usize,
                                            height: *height as usize,
                                            bytes: std::borrow::Cow::Owned(rgba_data),
                                        };
                                        clipboard
                                            .set_image(img)
                                            .map_err(|e| format!("Failed to set clipboard image: {}", e))?;
                                    }
                                    Err(e) => {
                                        return Err(format!("Failed to decode image data: {}", e));
                                    }
                                }
                            } else {
                                return Err("Invalid image dimensions: width and height must be greater than 0".to_string());
                            }
                        } else {
                            return Err("Invalid base64 image data".to_string());
                        }
                    }
                    ClipboardContentType::Html { html, plain_text } => {
                        // Try HTML first, fallback to plain text
                        if let Some(plain) = plain_text {
                            if clipboard.set_html(html, Some(plain)).is_err() {
                                clipboard
                                    .set_text(plain.clone())
                                    .map_err(|_| "Failed to set clipboard text")?;
                            }
                        } else {
                            clipboard
                                .set_text(html.clone())
                                .map_err(|_| "Failed to set clipboard text")?;
                        }
                    }
                    ClipboardContentType::Files(paths) => {
                        // Convert string paths to PathBuf
                        let _path_bufs: Vec<std::path::PathBuf> =
                            paths.iter().map(std::path::PathBuf::from).collect();
                        clipboard
                            .set_text(paths.join("\n"))
                            .map_err(|_| "Failed to set file paths as text")?;
                    }
                    ClipboardContentType::Other { data, .. } => {
                        // For other types, try to decode as text or set as base64
                        if let Ok(decoded) = BASE64_STANDARD.decode(data) {
                            if let Ok(text) = String::from_utf8(decoded) {
                                clipboard
                                    .set_text(text)
                                    .map_err(|_| "Failed to set clipboard text")?;
                            } else {
                                clipboard
                                    .set_text(data.clone())
                                    .map_err(|_| "Failed to set clipboard text")?;
                            }
                        } else {
                            clipboard
                                .set_text(data.clone())
                                .map_err(|_| "Failed to set clipboard text")?;
                        }
                    }
                }
                Ok(())
            })
            .await;

            match result {
                Ok(Ok(())) => Ok(true),
                Ok(Err(_)) => Ok(false),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    async fn save_history(&self) -> io::Result<()> {
        let history = self.history.lock().await;
        self.storage.save_history(&history).await
    }
}
