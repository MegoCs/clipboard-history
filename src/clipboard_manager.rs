use crate::clipboard_item::ClipboardItem;
use crate::storage::Storage;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
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

    #[cfg(test)]
    #[allow(dead_code)]
    pub async fn new_with_storage(storage: Storage) -> io::Result<Self> {
        let history = Arc::new(Mutex::new(storage.load_history().await?));
        Ok(Self { history, storage })
    }

    #[cfg(test)]
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

    async fn save_history(&self) -> io::Result<()> {
        let history = self.history.lock().await;
        self.storage.save_history(&history).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clipboard_manager_creation() {
        let manager = ClipboardManager::new_empty();
        assert_eq!(manager.get_history_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_item() {
        let manager = ClipboardManager::new_empty();
        let content = "Test content".to_string();

        let result = manager.add_item(content.clone()).await;
        assert!(result.is_ok());

        let history = manager.get_history().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, content);
    }

    #[tokio::test]
    async fn test_duplicate_prevention() {
        let manager = ClipboardManager::new_empty();
        let content = "Duplicate content".to_string();

        manager.add_item(content.clone()).await.unwrap();
        manager.add_item(content.clone()).await.unwrap(); // Should be ignored

        let history = manager.get_history().await;
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let manager = ClipboardManager::new_empty();

        manager.add_item("Hello World".to_string()).await.unwrap();
        manager
            .add_item("Rust programming".to_string())
            .await
            .unwrap();
        manager
            .add_item("Clipboard manager".to_string())
            .await
            .unwrap();

        let results = manager.search_history("rust").await;
        assert_eq!(results.len(), 1);
        assert!(results[0].1.content.contains("Rust"));
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        let manager = ClipboardManager::new_empty();

        manager.add_item("Hello World".to_string()).await.unwrap();
        manager.add_item("Help wanted".to_string()).await.unwrap();

        let results = manager.fuzzy_search_history("helo").await; // typo
        assert!(!results.is_empty());
        // Should find "Hello World" despite the typo
        assert!(results
            .iter()
            .any(|(_, item, _)| item.content.contains("Hello")));
    }

    #[tokio::test]
    async fn test_clear_history() {
        let manager = ClipboardManager::new_empty();

        manager.add_item("Item 1".to_string()).await.unwrap();
        manager.add_item("Item 2".to_string()).await.unwrap();

        assert_eq!(manager.get_history_count().await, 2);

        manager.clear_history().await.unwrap();
        assert_eq!(manager.get_history_count().await, 0);
    }

    #[tokio::test]
    async fn test_get_item_by_index() {
        let manager = ClipboardManager::new_empty();

        manager.add_item("First item".to_string()).await.unwrap();
        manager.add_item("Second item".to_string()).await.unwrap();

        let item = manager.get_item_by_index(0).await;
        assert!(item.is_some());
        assert_eq!(item.unwrap().content, "Second item"); // Most recent first

        let item = manager.get_item_by_index(1).await;
        assert!(item.is_some());
        assert_eq!(item.unwrap().content, "First item");

        let item = manager.get_item_by_index(999).await;
        assert!(item.is_none());
    }
}
