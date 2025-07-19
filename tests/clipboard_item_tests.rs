use clipboard_history::clipboard_item::ClipboardItem;

#[test]
fn test_clipboard_item_creation() {
    let content = "Test clipboard content".to_string();
    let item = ClipboardItem::new(content.clone(), 1);

    assert_eq!(item.content, content);
    assert_eq!(item.id, 1);
    assert!(item.timestamp > 0);
}

#[test]
fn test_clipboard_item_preview() {
    let long_content = "This is a very long clipboard content that should be truncated when displayed as a preview to the user".to_string();
    let item = ClipboardItem::new(long_content, 1);

    let preview = item.preview(20);
    // The preview now includes size info for truncated content
    assert!(preview.starts_with("This is a very long "));
    assert!(preview.contains("B"));
}

#[test]
fn test_clipboard_item_preview_short_content() {
    let short_content = "Short".to_string();
    let item = ClipboardItem::new(short_content.clone(), 1);

    let preview = item.preview(20);
    assert_eq!(preview, short_content);
}

#[test]
fn test_smart_preview() {
    let long_content = "This is a very long clipboard content that should be truncated when displayed as a preview to the user".to_string();
    let item = ClipboardItem::new(long_content, 1);

    let smart_preview = item.smart_preview(20);
    assert!(smart_preview.starts_with("This is a very long "));
    assert!(smart_preview.contains("Text"));
    assert!(smart_preview.contains("B"));
}

#[test]
fn test_content_analysis() {
    // Test JSON detection with content that will be truncated
    let json_content = format!(
        "{{{}, \"large_field\": \"{}\"}}",
        "\"key\": \"value\"",
        "x".repeat(200)
    );
    let json_item = ClipboardItem::new(json_content, 1);
    let preview = json_item.smart_preview(50);
    assert!(preview.contains("JSON"));

    // Test URL detection with content that will be truncated
    let url_content = format!("https://example.com {}", "x".repeat(200));
    let url_item = ClipboardItem::new(url_content, 1);
    let preview = url_item.smart_preview(50);
    assert!(preview.contains("URL"));

    // Test multi-line detection
    let multiline_content =
        "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11"
            .to_string();
    let multiline_item = ClipboardItem::new(multiline_content, 1);
    let preview = multiline_item.smart_preview(20);
    assert!(preview.contains("Multi-line"));
}

#[test]
fn test_format_content_size() {
    let small_item = ClipboardItem::new("Hello".to_string(), 1);
    assert_eq!(small_item.format_content_size(), "5 B");

    let kb_content = "x".repeat(1536); // 1.5 KB
    let kb_item = ClipboardItem::new(kb_content, 1);
    assert_eq!(kb_item.format_content_size(), "1.5 KB");

    let mb_content = "x".repeat(1572864); // 1.5 MB
    let mb_item = ClipboardItem::new(mb_content, 1);
    assert_eq!(mb_item.format_content_size(), "1.5 MB");
}

#[test]
fn test_formatted_timestamp() {
    let item = ClipboardItem::new("test".to_string(), 1);
    let formatted = item.formatted_timestamp();

    // Should be in format YYYY-MM-DD HH:MM:SS or fallback format
    assert!(!formatted.is_empty());
    assert!(formatted.contains("-") || formatted.contains("ts:"));
}
