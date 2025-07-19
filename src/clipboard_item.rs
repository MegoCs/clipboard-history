use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, TimeZone};

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
