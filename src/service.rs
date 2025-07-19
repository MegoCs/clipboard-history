use crate::clipboard_item::ClipboardItem;
use crate::clipboard_manager::ClipboardManager;
use crate::monitor::{ClipboardEvent, ClipboardMonitor};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Core service that provides all clipboard management functionality
/// This is completely UI-agnostic and can be used by any interface (console, desktop, web, etc.)
pub struct ClipboardService {
    manager: Arc<ClipboardManager>,
    monitor: Option<ClipboardMonitor>,
}

impl ClipboardService {
    /// Create a new clipboard service instance
    pub async fn new() -> io::Result<Self> {
        let manager = Arc::new(ClipboardManager::new().await?);
        let monitor = ClipboardMonitor::new(Arc::clone(&manager));
        
        Ok(Self {
            manager,
            monitor: Some(monitor),
        })
    }

    /// Start background clipboard monitoring
    /// Returns a receiver for clipboard events
    pub fn start_monitoring(&mut self) -> Option<broadcast::Receiver<ClipboardEvent>> {
        if let Some(monitor) = self.monitor.take() {
            let event_receiver = monitor.subscribe();
            let monitor_task = tokio::spawn(async move {
                monitor.start_monitoring().await;
            });
            
            // Store the task handle if needed for cleanup
            // For now, we'll let it run until the service is dropped
            std::mem::forget(monitor_task);
            
            Some(event_receiver)
        } else {
            None
        }
    }

    /// Get the current clipboard history
    pub async fn get_history(&self) -> Vec<ClipboardItem> {
        self.manager.get_history().await
    }

    /// Get the count of items in history
    pub async fn get_history_count(&self) -> usize {
        self.manager.get_history_count().await
    }

    /// Search clipboard history with exact text matching
    pub async fn search(&self, query: &str) -> Vec<(usize, ClipboardItem)> {
        self.manager.search_history(query).await
    }

    /// Search clipboard history with fuzzy matching
    pub async fn fuzzy_search(&self, query: &str) -> Vec<(usize, ClipboardItem, i64)> {
        self.manager.fuzzy_search_history(query).await
    }

    /// Clear all clipboard history
    pub async fn clear_history(&self) -> io::Result<()> {
        self.manager.clear_history().await
    }

    /// Copy a specific item back to the system clipboard
    pub async fn copy_to_clipboard(&self, index: usize) -> io::Result<bool> {
        self.manager.copy_item_to_clipboard(index).await
    }

    /// Get the storage file path
    pub fn get_storage_path(&self) -> &PathBuf {
        self.manager.get_storage_path()
    }
}

/// Search result wrapper
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub index: usize,
    pub item: ClipboardItem,
    pub score: Option<i64>, // None for exact text search, Some(score) for fuzzy search
}

impl ClipboardService {
    /// Unified search method that returns both exact and fuzzy results
    pub async fn search_unified(&self, query: &str) -> (Vec<SearchResult>, Vec<SearchResult>) {
        let exact_results = self.search(query).await;
        let fuzzy_results = self.fuzzy_search(query).await;

        let exact = exact_results
            .into_iter()
            .map(|(index, item)| SearchResult {
                index,
                item,
                score: None,
            })
            .collect();

        let fuzzy = fuzzy_results
            .into_iter()
            .map(|(index, item, score)| SearchResult {
                index,
                item,
                score: Some(score),
            })
            .collect();

        (exact, fuzzy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let service = ClipboardService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test] 
    async fn test_service_operations() {
        // Create service with empty manager for testing
        let manager = Arc::new(ClipboardManager::new_empty());
        let service = ClipboardService {
            manager: Arc::clone(&manager),
            monitor: None,
        };
        
        // Test adding items through the manager
        assert!(manager.add_item("Test item 1".to_string()).await.is_ok());
        assert!(manager.add_item("Test item 2".to_string()).await.is_ok());
        
        // Test getting history
        let history = service.get_history().await;
        assert_eq!(history.len(), 2);
        assert_eq!(service.get_history_count().await, 2);
        
        // Test search
        let results = service.search("Test").await;
        assert_eq!(results.len(), 2);
        
        // Test fuzzy search
        let fuzzy_results = service.fuzzy_search("test").await;
        assert_eq!(fuzzy_results.len(), 2);
        
        // Test unified search
        let (exact, fuzzy) = service.search_unified("Test").await;
        assert!(!exact.is_empty() || !fuzzy.is_empty());
    }
}
