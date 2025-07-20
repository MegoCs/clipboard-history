use clipboard_history::clipboard_item::{ClipboardContentType, ClipboardItem};
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
    let item = ClipboardItem::new_text(content.clone());

    let result = manager.add_clipboard_item(item).await;
    assert!(result.is_ok());

    let history = manager.get_history().await;
    assert_eq!(history.len(), 1);
    if let ClipboardContentType::Text(text) = &history[0].content {
        assert_eq!(text, &content);
    } else {
        panic!("Expected Text content type");
    }
}

#[tokio::test]
async fn test_duplicate_prevention() {
    let manager = ClipboardManager::new_empty();
    let content = "Duplicate content".to_string();
    let item1 = ClipboardItem::new_text(content.clone());
    let item2 = ClipboardItem::new_text(content.clone());

    manager.add_clipboard_item(item1).await.unwrap();
    manager.add_clipboard_item(item2).await.unwrap(); // Should be ignored due to hash

    let history = manager.get_history().await;
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_search_functionality() {
    let manager = ClipboardManager::new_empty();

    manager
        .add_clipboard_item(ClipboardItem::new_text("Hello World".to_string()))
        .await
        .unwrap();
    manager
        .add_clipboard_item(ClipboardItem::new_text("Rust programming".to_string()))
        .await
        .unwrap();
    manager
        .add_clipboard_item(ClipboardItem::new_text("Clipboard manager".to_string()))
        .await
        .unwrap();

    let results = manager.search_history("rust").await;
    assert_eq!(results.len(), 1);
    // Check if the item contains "Rust" using display_content
    let searchable = results[0].1.display_content();
    assert!(searchable.contains("Rust"));
}

#[tokio::test]
async fn test_fuzzy_search() {
    let manager = ClipboardManager::new_empty();

    manager
        .add_clipboard_item(ClipboardItem::new_text("Hello World".to_string()))
        .await
        .unwrap();
    manager
        .add_clipboard_item(ClipboardItem::new_text("Help wanted".to_string()))
        .await
        .unwrap();

    let results = manager.fuzzy_search_history("helo").await; // typo
    assert!(!results.is_empty());
    // Should find "Hello World" despite the typo
    assert!(results.iter().any(|(_, item, _)| {
        let searchable = item.display_content();
        searchable.contains("Hello")
    }));
}

#[tokio::test]
async fn test_clear_history() {
    let manager = ClipboardManager::new_empty();

    manager
        .add_clipboard_item(ClipboardItem::new_text("Item 1".to_string()))
        .await
        .unwrap();
    manager
        .add_clipboard_item(ClipboardItem::new_text("Item 2".to_string()))
        .await
        .unwrap();

    assert_eq!(manager.get_history_count().await, 2);

    manager.clear_history().await.unwrap();
    assert_eq!(manager.get_history_count().await, 0);
}

#[tokio::test]
async fn test_history_access() {
    let manager = ClipboardManager::new_empty();

    manager
        .add_clipboard_item(ClipboardItem::new_text("First item".to_string()))
        .await
        .unwrap();
    manager
        .add_clipboard_item(ClipboardItem::new_text("Second item".to_string()))
        .await
        .unwrap();

    let history = manager.get_history().await;
    assert_eq!(history.len(), 2);

    // Most recent first - check using display_content
    let first_searchable = history[0].display_content();
    let second_searchable = history[1].display_content();
    assert!(first_searchable.contains("Second item"));
    assert!(second_searchable.contains("First item"));
}

#[tokio::test]
async fn test_content_size_limit() {
    let manager = ClipboardManager::new_empty();

    // Test normal content
    let normal_content = "x".repeat(1000);
    let normal_item = ClipboardItem::new_text(normal_content);
    let result = manager.add_clipboard_item(normal_item).await;
    assert!(result.is_ok());

    // Test oversized content (over 10MB limit)
    let oversized_content = "x".repeat(10_000_001);
    let oversized_item = ClipboardItem::new_text(oversized_content);
    let result = manager.add_clipboard_item(oversized_item).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("Content too large"));
}

#[tokio::test]
async fn test_usage_stats() {
    let manager = ClipboardManager::new_empty();

    manager
        .add_clipboard_item(ClipboardItem::new_text("Small".to_string()))
        .await
        .unwrap();
    manager
        .add_clipboard_item(ClipboardItem::new_text("Medium content here".to_string()))
        .await
        .unwrap();
    manager
        .add_clipboard_item(ClipboardItem::new_text("x".repeat(1000)))
        .await
        .unwrap(); // Large content

    let (item_count, total_size, avg_size, largest_item) = manager.get_usage_stats().await;

    assert_eq!(item_count, 3);
    assert!(total_size > 1000);
    assert!(largest_item >= 1000);
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
