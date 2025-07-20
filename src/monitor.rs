use base64::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::clipboard_item::{ClipboardContentType, ClipboardItem, ImageFormat};
use crate::clipboard_manager::ClipboardManager;

#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    ItemAdded,
    Error,
    Started,
}

pub struct ClipboardMonitor {
    manager: Arc<ClipboardManager>,
    poll_interval: Duration,
    event_sender: broadcast::Sender<ClipboardEvent>,
}

impl ClipboardMonitor {
    pub fn new(manager: Arc<ClipboardManager>) -> Self {
        let (event_sender, _) = broadcast::channel(100);

        Self {
            manager,
            poll_interval: Duration::from_millis(500),
            event_sender,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ClipboardEvent> {
        self.event_sender.subscribe()
    }

    #[allow(dead_code)]
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub async fn start_monitoring(&self) {
        let mut last_content_hash = String::new();

        // Notify that monitoring has started
        let _ = self.event_sender.send(ClipboardEvent::Started);

        loop {
            let content_result = self.get_clipboard_content().await;

            match content_result {
                Ok(clipboard_item) => {
                    // Create a hash of the content to detect changes
                    let content_hash = self.create_content_hash(&clipboard_item);

                    if !content_hash.is_empty() && content_hash != last_content_hash {
                        match self.manager.add_clipboard_item(clipboard_item).await {
                            Ok(()) => {
                                let _ = self
                                    .event_sender
                                    .send(ClipboardEvent::ItemAdded);
                            }
                            Err(_) => {
                                let _ = self.event_sender.send(ClipboardEvent::Error);
                            }
                        }
                        last_content_hash = content_hash;
                    }
                }
                Err(_) => {
                    let _ = self.event_sender.send(ClipboardEvent::Error);
                }
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Create a hash representation of clipboard content for change detection
    fn create_content_hash(&self, item: &ClipboardItem) -> String {
        match &item.content {
            ClipboardContentType::Text(text) => text.clone(),
            ClipboardContentType::Image {
                data,
                format,
                width,
                height,
            } => {
                format!("img:{}:{:?}:{}x{}", data.len(), format, width, height)
            }
            ClipboardContentType::Html { html, .. } => format!("html:{}", html.len()),
            ClipboardContentType::Files(files) => format!("files:{}", files.join("|")),
            ClipboardContentType::Other { content_type, data } => {
                format!("other:{}:{}", content_type, data.len())
            }
        }
    }

    async fn get_clipboard_content(&self) -> Result<ClipboardItem, String> {
        let result = tokio::task::spawn_blocking(|| {
            let mut clipboard =
                arboard::Clipboard::new().map_err(|_| "Failed to access clipboard")?;

            // Try to get image first (images have higher priority)
            if let Ok(image_data) = clipboard.get_image() {
                let width = image_data.width as u32;
                let height = image_data.height as u32;

                // Convert RGBA to PNG bytes for storage
                let png_data = Self::rgba_to_png(&image_data.bytes, width, height)
                    .map_err(|_| "Failed to encode image data")?;

                return Ok(ClipboardContentType::Image {
                    data: BASE64_STANDARD.encode(&png_data),
                    format: ImageFormat::Png,
                    width,
                    height,
                });
            }

            // Try to get HTML if available (not supported by arboard 3.6)
            // if let Ok(html) = clipboard.get_html() {
            //     let plain_text = clipboard.get_text().ok();
            //     return Ok(ClipboardContentType::Html { html, plain_text });
            // }

            // Try to get text
            if let Ok(text) = clipboard.get_text() {
                if !text.trim().is_empty() {
                    return Ok(ClipboardContentType::Text(text));
                }
            }

            Err("No supported clipboard content found")
        })
        .await;

        match result {
            Ok(Ok(content)) => {
                // Create a new ClipboardItem with the appropriate constructor
                let item = match content {
                    ClipboardContentType::Text(text) => ClipboardItem::new_text(text),
                    ClipboardContentType::Image {
                        data,
                        format,
                        width,
                        height,
                    } => {
                        // Convert base64 string back to bytes
                        if let Ok(decoded_data) = BASE64_STANDARD.decode(&data) {
                            ClipboardItem::new_image(decoded_data, format, width, height)
                        } else {
                            return Err("Failed to decode image data".to_string());
                        }
                    }
                    ClipboardContentType::Html { html, plain_text } => {
                        ClipboardItem::new_html(html, plain_text)
                    }
                    ClipboardContentType::Files(files) => ClipboardItem::new_files(files),
                    ClipboardContentType::Other { content_type, data } => {
                        ClipboardItem::new_other(content_type, data)
                    }
                };
                Ok(item)
            }
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(format!("Clipboard access error: {e}")),
        }
    }

    /// Convert RGBA bytes to PNG format
    fn rgba_to_png(
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use image::{ImageBuffer, Rgba};

        let img_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba_data)
            .ok_or("Failed to create image buffer")?;

        let mut png_data = Vec::new();
        img_buffer.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png,
        )?;
        Ok(png_data)
    }

    /// Convert PNG bytes back to RGBA format
    pub fn png_to_rgba(
        png_data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use image::ImageReader;
        
        let reader = ImageReader::new(std::io::Cursor::new(png_data))
            .with_guessed_format()
            .map_err(|e| format!("Failed to read image format: {}", e))?;
            
        let img = reader.decode()
            .map_err(|e| format!("Failed to decode image: {}", e))?;
            
        let rgba_img = img.to_rgba8();
        Ok(rgba_img.into_raw())
    }
}
