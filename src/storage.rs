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

    pub fn get_data_file_path(&self) -> &PathBuf {
        &self.data_file
    }

    pub async fn load_history(&self) -> io::Result<VecDeque<ClipboardItem>> {
        println!("Looking for history at: {:?}", self.data_file);
        
        if self.data_file.exists() {
            let content = fs::read_to_string(&self.data_file)?;
            if let Ok(loaded) = serde_json::from_str::<VecDeque<ClipboardItem>>(&content) {
                println!("Loaded {} items", loaded.len());
                return Ok(loaded);
            }
        }
        
        println!("No existing history found, starting fresh");
        Ok(VecDeque::new())
    }
    
    pub async fn save_history(&self, history: &VecDeque<ClipboardItem>) -> io::Result<()> {
        let json = serde_json::to_string_pretty(history)?;
        fs::write(&self.data_file, json)?;
        println!("Saved {} items", history.len());
        Ok(())
    }
}
