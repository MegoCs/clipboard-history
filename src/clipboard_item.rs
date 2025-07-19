use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub content: String,
    pub timestamp: u64,
    pub id: usize,
}

impl ClipboardItem {
    pub fn new(content: String, id: usize) -> Self {
        Self {
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            id,
        }
    }

    #[allow(dead_code)] // Used by tests and might be used by future UI implementations
    pub fn preview(&self, max_chars: usize) -> String {
        if self.content.len() <= max_chars {
            self.content.clone()
        } else {
            let truncated = self.content.chars().take(max_chars).collect::<String>();
            format!("{} [{}...]", truncated, self.format_content_size())
        }
    }

    /// Get a smart preview that shows content type and size for large entries
    pub fn smart_preview(&self, max_chars: usize) -> String {
        let content_info = self.analyze_content();

        if self.content.len() <= max_chars {
            self.content.clone()
        } else {
            let truncated = self.content.chars().take(max_chars).collect::<String>();
            format!(
                "{} [{}, {}...]",
                truncated,
                content_info,
                self.format_content_size()
            )
        }
    }

    /// Format content size in human-readable format
    pub fn format_content_size(&self) -> String {
        let size = self.content.len();
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        }
    }

    /// Analyze content type for better preview
    fn analyze_content(&self) -> &'static str {
        let content = &self.content;

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
        } else {
            "Text"
        }
    }

    pub fn formatted_timestamp(&self) -> String {
        if let Some(datetime) = Utc.timestamp_opt(self.timestamp as i64, 0).single() {
            let local_time: DateTime<chrono::Local> = datetime.into();
            local_time.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            format!("ts:{}", self.timestamp)
        }
    }
}
