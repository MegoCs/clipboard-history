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

    pub fn preview(&self, max_chars: usize) -> String {
        self.content.chars().take(max_chars).collect::<String>()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_item_creation() {
        let content = "Test clipboard content".to_string();
        let item = ClipboardItem::new(content.clone(), 1);

        assert_eq!(item.content, content);
        assert_eq!(item.id, 1);
        assert!(item.timestamp > 0);
    }

    #[test]
    fn test_clipboard_item_preview() {
        let long_content = "This is a very long clipboard content that should be truncated when displayed as a preview to the user".to_string();
        let item = ClipboardItem::new(long_content, 1);

        let preview = item.preview(20);
        assert_eq!(preview.len(), 20);
        assert_eq!(preview, "This is a very long ");
    }

    #[test]
    fn test_clipboard_item_preview_short_content() {
        let short_content = "Short".to_string();
        let item = ClipboardItem::new(short_content.clone(), 1);

        let preview = item.preview(20);
        assert_eq!(preview, short_content);
    }

    #[test]
    fn test_formatted_timestamp() {
        let item = ClipboardItem::new("test".to_string(), 1);
        let formatted = item.formatted_timestamp();

        // Should be in format YYYY-MM-DD HH:MM:SS or fallback format
        assert!(!formatted.is_empty());
        assert!(formatted.contains("-") || formatted.contains("ts:"));
    }
}
