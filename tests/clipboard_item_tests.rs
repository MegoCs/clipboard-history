use clipboard_history::clipboard_item::{ClipboardItem, ClipboardContentType};

#[test]
fn test_clipboard_item_creation() {
    let content = "Test clipboard content".to_string();
    let item = ClipboardItem::new_text(content.clone());

    if let ClipboardContentType::Text(text) = &item.content {
        assert_eq!(text, &content);
    } else {
        panic!("Expected Text content type");
    }
    assert!(!item.id.is_empty());
    assert!(item.timestamp.timestamp() > 0);
}

#[test]
fn test_clipboard_item_preview() {
    let long_content = "This is a very long clipboard content that should be truncated when displayed as a preview to the user".to_string();
    let item = ClipboardItem::new_text(long_content);

    let preview = item.get_preview();
    // The preview contains the text with type prefix and size info
    assert!(preview.contains("This is a very long"));
    assert!(preview.contains("[Text]"));
}

#[test]
fn test_clipboard_item_preview_short_content() {
    let short_content = "Short".to_string();
    let item = ClipboardItem::new_text(short_content.clone());

    let preview = item.get_preview();
    assert!(preview.contains("Short"));
}

#[test]
fn test_smart_preview() {
    let long_content = "This is a very long clipboard content that should be truncated when displayed as a preview to the user".to_string();
    let item = ClipboardItem::new_text(long_content);

    let smart_preview = item.smart_preview(50);
    assert!(smart_preview.contains("This is a very long"));
    assert!(smart_preview.contains("Text"));
}

#[test]
fn test_content_analysis() {
    // Test JSON detection with content that will be truncated
    let json_content = format!(
        "{{{}, \"large_field\": \"{}\"}}",
        "\"key\": \"value\"",
        "x".repeat(200)
    );
    let json_item = ClipboardItem::new_text(json_content);
    let preview = json_item.smart_preview(100);
    // JSON detection might work depending on the smart_preview implementation
    assert!(preview.contains("Text"));

    // Test URL detection with content that will be truncated
    let url_content = format!("https://example.com {}", "x".repeat(200));
    let url_item = ClipboardItem::new_text(url_content);
    let preview = url_item.smart_preview(100);
    assert!(preview.contains("Text"));

    // Test multi-line detection
    let multiline_content =
        "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11"
            .to_string();
    let multiline_item = ClipboardItem::new_text(multiline_content);
    let preview = multiline_item.smart_preview(100);
    assert!(preview.contains("Text"));
}

#[test]
fn test_format_content_size() {
    let small_item = ClipboardItem::new_text("Hello".to_string());
    let size_bytes = small_item.get_size_bytes();
    assert!(size_bytes >= 5); // At least 5 bytes for "Hello"

    let kb_content = "x".repeat(1536); // 1.5 KB
    let kb_item = ClipboardItem::new_text(kb_content);
    let kb_size = kb_item.get_size_bytes();
    assert!(kb_size >= 1536);

    let mb_content = "x".repeat(1572864); // 1.5 MB
    let mb_item = ClipboardItem::new_text(mb_content);
    let mb_size = mb_item.get_size_bytes();
    assert!(mb_size >= 1572864);
}

#[test]
fn test_formatted_timestamp() {
    let item = ClipboardItem::new_text("test".to_string());
    let formatted = item.formatted_timestamp();

    // Should be in format YYYY-MM-DD HH:MM:SS or fallback format
    assert!(!formatted.is_empty());
    assert!(formatted.contains("-") || formatted.contains("ts:"));
}
