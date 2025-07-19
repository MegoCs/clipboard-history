use crate::clipboard_item::ClipboardItem;
use crate::storage::Storage;
use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

const MAX_HISTORY_SIZE: usize = 1000;

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

    pub async fn add_item(&self, content: String) -> io::Result<()> {
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

    pub async fn get_item_by_index(&self, index: usize) -> Option<ClipboardItem> {
        let history = self.history.lock().await;
        history.get(index).cloned()
    }

    pub async fn clear_history(&self) -> io::Result<()> {
        let mut history = self.history.lock().await;
        history.clear();
        drop(history);
        self.save_history().await
    }

    pub async fn search_history(&self, query: &str) -> Vec<(usize, ClipboardItem)> {
        let history = self.history.lock().await;
        history
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.content.to_lowercase().contains(&query.to_lowercase())
            })
            .map(|(idx, item)| (idx, item.clone()))
            .collect()
    }

    pub fn get_storage_path(&self) -> &std::path::PathBuf {
        self.storage.get_data_file_path()
    }

    async fn save_history(&self) -> io::Result<()> {
        let history = self.history.lock().await;
        self.storage.save_history(&history).await
    }
}
