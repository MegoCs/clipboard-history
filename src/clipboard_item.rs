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

    /// Get a preview of the content with default length
    pub fn get_preview(&self) -> String {
        self.smart_preview(100)
    }

    /// Get content type as string for display
    pub fn content_type_name(&self) -> &'static str {
        match &self.content {
            ClipboardContentType::Text(_) => "Text",
            ClipboardContentType::Image { .. } => "Image",
            ClipboardContentType::Html { .. } => "HTML",
            ClipboardContentType::Files(_) => "Files",
            ClipboardContentType::Other { .. } => "Binary",
        }
    }

    #[allow(dead_code)] // Used by tests and might be used by future UI implementations
    pub fn preview(&self, max_chars: usize) -> String {
        let content_str = self.display_content();
        if content_str.len() <= max_chars {
            format!("[{}] {}", self.content_type_name(), content_str)
        } else {
            let truncated = content_str.chars().take(max_chars).collect::<String>();
            format!(
                "[{}] {} [{}...]",
                self.content_type_name(),
                truncated,
                self.format_content_size()
            )
        }
    }

    /// Get a smart preview that shows content type and size for large entries
    pub fn smart_preview(&self, max_chars: usize) -> String {
        let content_info = self.analyze_content();
        let content_str = self.display_content();

        if content_str.len() <= max_chars {
            format!("[{}] {}", self.content_type_name(), content_str)
        } else {
            let truncated = content_str.chars().take(max_chars).collect::<String>();
            format!(
                "[{}] {} [{}, {}...]",
                self.content_type_name(),
                truncated,
                content_info,
                self.format_content_size()
            )
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

    /// Format content size in human-readable format
    pub fn format_content_size(&self) -> String {
        let size = self.estimate_size();
        if size < 1024 {
            format!("{size} B")
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
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

    /// Analyze content type for better preview
    fn analyze_content(&self) -> &'static str {
        match &self.content {
            ClipboardContentType::Text(content) => {
                // Check for common data patterns
                if content.trim().starts_with('{') && content.trim().ends_with('}') {
                    "JSON"
                } else if content.trim().starts_with('<') && content.trim().ends_with('>') {
                    "HTML/XML"
                } else if content.contains("http://") || content.contains("https://") {
                    "URL/Link"
                } else if content.lines().count() > 10 {
                    "Multi-line"
                } else if content
                    .chars()
                    .all(|c| c.is_ascii_digit() || c.is_whitespace() || c == '.' || c == '-')
                {
                    "Numeric"
                } else if !content.is_ascii() {
                    "Unicode/Emoji"
                } else {
                    "Text"
                }
            }
            ClipboardContentType::Image { width, height, .. } => {
                if *width > 1920 || *height > 1080 {
                    "Large Image"
                } else {
                    "Image"
                }
            }
            ClipboardContentType::Html { .. } => "Rich Text",
            ClipboardContentType::Files(files) => {
                if files.len() == 1 {
                    "Single File"
                } else {
                    "Multiple Files"
                }
            }
            ClipboardContentType::Other { .. } => "Binary Data",
        }
    }

    pub fn formatted_timestamp(&self) -> String {
        let local_time: DateTime<chrono::Local> = self.timestamp.into();
        local_time.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
