use crate::clipboard_item::ClipboardItem;
use crate::storage::Storage;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

const MAX_HISTORY_SIZE: usize = 1000;
const MAX_CONTENT_SIZE: usize = 10_000_000; // 10MB limit for individual entries
const MAX_PREVIEW_LENGTH: usize = 200; // Default preview length for large entries

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

    pub async fn add_item(&self, content: String) -> io::Result<()> {
        // Check content size limit
        if content.len() > MAX_CONTENT_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Content too large: {} bytes (max: {} bytes)", 
                    content.len(), 
                    MAX_CONTENT_SIZE
                )
            ));
        }

        let mut history = self.history.lock().await;

        // Skip duplicates
        if let Some(last) = history.front() {
            if last.content == content {
                return Ok(());
            }
        }

        let item = ClipboardItem::new(content, history.len());
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

    pub async fn get_history_count(&self) -> usize {
        let history = self.history.lock().await;
        history.len()
    }

    pub async fn clear_history(&self) -> io::Result<()> {
        let mut history = self.history.lock().await;
        history.clear();
        drop(history);
        self.save_history().await
    }

    pub async fn search_history(&self, query: &str) -> Vec<(usize, ClipboardItem)> {
        let history = self.history.lock().await;

        // Simple text-based search (case-insensitive contains)
        let text_matches: Vec<(usize, ClipboardItem)> = history
            .iter()
            .enumerate()
            .filter(|(_, item)| item.content.to_lowercase().contains(&query.to_lowercase()))
            .map(|(idx, item)| (idx, item.clone()))
            .collect();

        text_matches
    }

    pub async fn fuzzy_search_history(&self, query: &str) -> Vec<(usize, ClipboardItem, i64)> {
        let history = self.history.lock().await;
        let matcher = SkimMatcherV2::default();

        let mut fuzzy_matches: Vec<(usize, ClipboardItem, i64)> = history
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                matcher
                    .fuzzy_match(&item.content, query)
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
            let content = item.content.clone();
            drop(history);

            // Use blocking task for clipboard operation
            let result = tokio::task::spawn_blocking(move || {
                use clipboard::ClipboardProvider;
                match clipboard::ClipboardContext::new() {
                    Ok(mut ctx) => ctx.set_contents(content).is_ok(),
                    Err(_) => false,
                }
            })
            .await;

            match result {
                Ok(success) => Ok(success),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    pub fn get_storage_path(&self) -> &std::path::PathBuf {
        self.storage.get_data_file_path()
    }

    /// Get the current content size limits
    pub fn get_content_limits(&self) -> (usize, usize, usize) {
        (MAX_CONTENT_SIZE, MAX_HISTORY_SIZE, MAX_PREVIEW_LENGTH)
    }

    /// Get total size of all clipboard content in bytes
    pub async fn get_total_content_size(&self) -> usize {
        let history = self.history.lock().await;
        history.iter().map(|item| item.content.len()).sum()
    }

    /// Get statistics about clipboard usage
    pub async fn get_usage_stats(&self) -> (usize, usize, usize, usize) {
        let history = self.history.lock().await;
        let item_count = history.len();
        let total_size = history.iter().map(|item| item.content.len()).sum();
        let avg_size = if item_count > 0 { total_size / item_count } else { 0 };
        let largest_item = history.iter().map(|item| item.content.len()).max().unwrap_or(0);
        
        (item_count, total_size, avg_size, largest_item)
    }

    async fn save_history(&self) -> io::Result<()> {
        let history = self.history.lock().await;
        self.storage.save_history(&history).await
    }
}
