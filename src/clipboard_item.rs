use base64::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardContentType {
    Text(String),
    Image {
        data: String, // Base64 encoded image data
        format: ImageFormat,
        width: u32,
        height: u32,
    },
    Html {
        html: String,
        plain_text: Option<String>, // Fallback plain text
    },
    Files(Vec<String>), // File paths
    Other {
        content_type: String,
        data: String, // Base64 encoded for binary data
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String, // Use UUID for better uniqueness
    pub content: ClipboardContentType,
    pub timestamp: DateTime<Utc>,
    pub content_hash: String, // Add content hash for deduplication
}

impl ClipboardItem {
    pub fn new(content: ClipboardContentType) -> Self {
        let id = Uuid::new_v4().to_string();
        let content_hash = Self::calculate_content_hash(&content);
        Self {
            id,
            content,
            timestamp: Utc::now(),
            content_hash,
        }
    }

    pub fn new_text(content: String) -> Self {
        Self::new(ClipboardContentType::Text(content))
    }

    pub fn new_image(data: Vec<u8>, format: ImageFormat, width: u32, height: u32) -> Self {
        let encoded_data = base64::prelude::BASE64_STANDARD.encode(&data);
        Self::new(ClipboardContentType::Image {
            data: encoded_data,
            format,
            width,
            height,
        })
    }

    pub fn new_html(html: String, plain_text: Option<String>) -> Self {
        Self::new(ClipboardContentType::Html { html, plain_text })
    }

    pub fn new_files(files: Vec<String>) -> Self {
        Self::new(ClipboardContentType::Files(files))
    }

    pub fn new_other(content_type: String, data: String) -> Self {
        Self::new(ClipboardContentType::Other { content_type, data })
    }

    /// Calculate hash for content deduplication
    fn calculate_content_hash(content: &ClipboardContentType) -> String {
        let mut hasher = DefaultHasher::new();
        match content {
            ClipboardContentType::Text(text) => text.hash(&mut hasher),
            ClipboardContentType::Image {
                data,
                format,
                width,
                height,
            } => {
                data.hash(&mut hasher);
                format.hash(&mut hasher);
                width.hash(&mut hasher);
                height.hash(&mut hasher);
            }
            ClipboardContentType::Html { html, plain_text } => {
                html.hash(&mut hasher);
                plain_text.hash(&mut hasher);
            }
            ClipboardContentType::Files(files) => files.hash(&mut hasher),
            ClipboardContentType::Other { content_type, data } => {
                content_type.hash(&mut hasher);
                data.hash(&mut hasher);
            }
        }
        hasher.finish().to_string()
    }

    /// Get the size in bytes for this clipboard item
    pub fn get_size_bytes(&self) -> usize {
        self.estimate_size()
    }

    /// Get clean preview without type prefix for search and display
    pub fn clean_preview(&self, max_chars: usize) -> String {
        let content_str = self.display_content();

        if content_str.len() <= max_chars {
            content_str
        } else {
            let truncated = content_str.chars().take(max_chars).collect::<String>();
            format!("{}...", truncated)
        }
    }

    /// Get display-friendly content string
    pub fn display_content(&self) -> String {
        match &self.content {
            ClipboardContentType::Text(text) => text.clone(),
            ClipboardContentType::Image {
                width,
                height,
                format,
                ..
            } => {
                format!("{width}x{height} {format:?} image")
            }
            ClipboardContentType::Html { plain_text, html } => {
                plain_text.as_ref().unwrap_or(html).clone()
            }
            ClipboardContentType::Files(files) => {
                if files.len() == 1 {
                    format!("File: {}", files[0])
                } else {
                    format!("{} files: {}", files.len(), files.join(", "))
                }
            }
            ClipboardContentType::Other { content_type, .. } => {
                format!("Binary data ({content_type})")
            }
        }
    }

    /// Estimate memory size of the content
    fn estimate_size(&self) -> usize {
        match &self.content {
            ClipboardContentType::Text(text) => text.len(),
            ClipboardContentType::Image { data, .. } => data.len(), // Base64 encoded size
            ClipboardContentType::Html { html, plain_text } => {
                html.len() + plain_text.as_ref().map_or(0, |t| t.len())
            }
            ClipboardContentType::Files(files) => files.iter().map(|f| f.len()).sum::<usize>(),
            ClipboardContentType::Other { content_type, data } => content_type.len() + data.len(),
        }
    }
}
