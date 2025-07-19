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
        // Simple timestamp formatting - you can enhance this later
        format!("ts:{}", self.timestamp)
    }
}
