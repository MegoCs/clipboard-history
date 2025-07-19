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

    /// Create a service instance with a provided manager (for testing)
    #[allow(dead_code)] // Used by tests
    pub fn new_with_manager(manager: Arc<ClipboardManager>) -> Self {
        Self {
            manager,
            monitor: None,
        }
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

    /// Get content size limits
    pub fn get_content_limits(&self) -> (usize, usize, usize) {
        self.manager.get_content_limits()
    }

    /// Get clipboard usage statistics
    pub async fn get_usage_stats(&self) -> (usize, usize, usize, usize) {
        self.manager.get_usage_stats().await
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
