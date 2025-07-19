use clipboard_history::clipboard_manager::ClipboardManager;

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
async fn test_history_access() {
    let manager = ClipboardManager::new_empty();

    manager.add_item("First item".to_string()).await.unwrap();
    manager.add_item("Second item".to_string()).await.unwrap();

    let history = manager.get_history().await;
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].content, "Second item"); // Most recent first
    assert_eq!(history[1].content, "First item");
}

#[tokio::test]
async fn test_content_size_limit() {
    let manager = ClipboardManager::new_empty();

    // Test normal content
    let normal_content = "x".repeat(1000);
    let result = manager.add_item(normal_content).await;
    assert!(result.is_ok());

    // Test oversized content (over 10MB limit)
    let oversized_content = "x".repeat(10_000_001);
    let result = manager.add_item(oversized_content).await;
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Content too large"));
}

#[tokio::test]
async fn test_usage_stats() {
    let manager = ClipboardManager::new_empty();

    manager.add_item("Small".to_string()).await.unwrap();
    manager.add_item("Medium content here".to_string()).await.unwrap();
    manager.add_item("x".repeat(1000)).await.unwrap(); // Large content

    let (item_count, total_size, avg_size, largest_item) = manager.get_usage_stats().await;
    
    assert_eq!(item_count, 3);
    assert!(total_size > 1000);
    assert_eq!(largest_item, 1000);
    assert!(avg_size > 0);
}

#[tokio::test]
async fn test_content_limits() {
    let manager = ClipboardManager::new_empty();
    let (max_content, max_history, max_preview) = manager.get_content_limits();
    
    assert_eq!(max_content, 10_000_000);
    assert_eq!(max_history, 1000);
    assert_eq!(max_preview, 200);
}
