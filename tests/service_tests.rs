use clipboard_history::clipboard_item::ClipboardItem;
use clipboard_history::clipboard_manager::ClipboardManager;
use clipboard_history::service::ClipboardService;
use std::sync::Arc;

#[tokio::test]
async fn test_service_creation() {
    let service = ClipboardService::new().await;
    assert!(service.is_ok());
}

#[tokio::test]
async fn test_service_operations() {
    // Create service with empty manager for testing
    let manager = Arc::new(ClipboardManager::new_empty());
    let service = ClipboardService::new_with_manager(manager.clone());

    // Test adding items through the manager
    assert!(manager
        .add_clipboard_item(ClipboardItem::new_text("Test item 1".to_string()))
        .await
        .is_ok());
    assert!(manager
        .add_clipboard_item(ClipboardItem::new_text("Test item 2".to_string()))
        .await
        .is_ok());

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

    // Test usage stats
    let (item_count, total_size, avg_size, largest_item) = service.get_usage_stats().await;
    assert_eq!(item_count, 2);
    assert!(total_size > 0);
    assert!(avg_size > 0);
    assert!(largest_item > 0);

    // Test content limits
    let (max_content, max_history, max_preview) = service.get_content_limits();
    assert!(max_content > 0);
    assert!(max_history > 0);
    assert!(max_preview > 0);
}
