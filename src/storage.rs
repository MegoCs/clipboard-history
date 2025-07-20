use crate::clipboard_item::ClipboardItem;
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Storage {
    data_file: PathBuf,
}

impl Storage {
    pub fn new() -> io::Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clipboard-history");

        fs::create_dir_all(&data_dir)?;
        let data_file = data_dir.join("history.json");

        Ok(Self { data_file })
    }

    // Public method for testing - allows specifying a custom file path
    #[allow(dead_code)] // Used by tests
    pub fn new_with_file(file_path: PathBuf) -> io::Result<Self> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self {
            data_file: file_path,
        })
    }

    pub async fn load_history(&self) -> io::Result<VecDeque<ClipboardItem>> {
        if self.data_file.exists() {
            let content = fs::read_to_string(&self.data_file)?;
            if let Ok(loaded) = serde_json::from_str::<VecDeque<ClipboardItem>>(&content) {
                return Ok(loaded);
            }
        }

        Ok(VecDeque::new())
    }

    pub async fn save_history(&self, history: &VecDeque<ClipboardItem>) -> io::Result<()> {
        let json = serde_json::to_string_pretty(history)?;
        fs::write(&self.data_file, json)?;
        Ok(())
    }
}
