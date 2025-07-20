use crate::clipboard_item::ClipboardItem;
use crate::clipboard_manager::ClipboardManager;
use crate::monitor::{ClipboardEvent, ClipboardMonitor};
use std::io;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Core service that provides all clipboard management functionality
/// This is completely UI-agnostic and can be used by any interface (console, desktop, web, etc.)
#[derive(Clone)]
pub struct ClipboardService {
    manager: Arc<ClipboardManager>,
    monitor: Option<Arc<ClipboardMonitor>>,
}

impl ClipboardService {
    /// Create a new clipboard service instance
    pub async fn new() -> io::Result<Self> {
        let manager = Arc::new(ClipboardManager::new().await?);
        let monitor = Arc::new(ClipboardMonitor::new(Arc::clone(&manager)));

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
        if let Some(monitor) = &self.monitor {
            let event_receiver = monitor.subscribe();
            let monitor_clone = Arc::clone(monitor);
            let monitor_task = tokio::spawn(async move {
                monitor_clone.start_monitoring().await;
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

    /// Search clipboard history with exact text matching
    pub async fn search(&self, query: &str) -> Vec<(usize, ClipboardItem)> {
        self.manager.search_history(query).await
    }

    /// Search clipboard history with fuzzy matching
    pub async fn fuzzy_search(&self, query: &str) -> Vec<(usize, ClipboardItem, i64)> {
        self.manager.fuzzy_search_history(query).await
    }

    /// Copy a specific item back to the system clipboard
    pub async fn copy_to_clipboard(&self, index: usize) -> io::Result<bool> {
        self.manager.copy_item_to_clipboard(index).await
    }
}

/// Search result wrapper
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub index: usize,
    pub item: ClipboardItem,
    #[allow(dead_code)] // May be used for future search result ranking features
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
