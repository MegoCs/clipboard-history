use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::clipboard_manager::ClipboardManager;

#[derive(Debug, Clone)]
pub enum ClipboardEvent {
    ItemAdded { preview: String },
    Error { message: String },
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
        let mut last_content = String::new();

        // Notify that monitoring has started
        let _ = self.event_sender.send(ClipboardEvent::Started);

        loop {
            let content = self.get_clipboard_content().await;

            if let Ok(content) = content {
                if !content.is_empty() && content != last_content {
                    // Create a more descriptive preview for large content
                    let preview = if content.len() > 200 {
                        let truncated = content[..200.min(content.len())].to_string();
                        format!("{} [{}...]", truncated, format_bytes(content.len()))
                    } else {
                        content[..50.min(content.len())].to_string()
                    };

                    match self.manager.add_item(content.clone()).await {
                        Ok(()) => {
                            let _ = self
                                .event_sender
                                .send(ClipboardEvent::ItemAdded { preview });
                        }
                        Err(e) => {
                            let _ = self.event_sender.send(ClipboardEvent::Error {
                                message: format!("Error adding clipboard item: {e}"),
                            });
                        }
                    }
                    last_content = content;
                }
            } else if let Err(e) = content {
                let _ = self.event_sender.send(ClipboardEvent::Error { message: e });
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    async fn get_clipboard_content(&self) -> Result<String, String> {
        let result = tokio::task::spawn_blocking(|| {
            let mut clipboard = arboard::Clipboard::new().map_err(|_| "Failed to access clipboard")?;
            clipboard.get_text().map_err(|_| "Failed to get clipboard text")
        })
        .await;

        match result {
            Ok(Ok(content)) => Ok(content),
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(format!("Clipboard access error: {e}")),
        }
    }
}

/// Format bytes in a human-readable format
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as usize, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
