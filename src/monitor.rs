use clipboard::ClipboardProvider;
use std::sync::Arc;
use std::time::Duration;

use crate::clipboard_manager::ClipboardManager;

pub struct ClipboardMonitor {
    manager: Arc<ClipboardManager>,
    poll_interval: Duration,
}

impl ClipboardMonitor {
    pub fn new(manager: Arc<ClipboardManager>) -> Self {
        Self {
            manager,
            poll_interval: Duration::from_millis(500),
        }
    }

    #[allow(dead_code)]
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub async fn start_monitoring(&self) {
        let mut last_content = String::new();

        loop {
            let content = self.get_clipboard_content().await;

            if let Ok(content) = content {
                if !content.is_empty() && content != last_content {
                    println!("New clipboard: {:?}", &content[..50.min(content.len())]);
                    if let Err(e) = self.manager.add_item(content.clone()).await {
                        eprintln!("Error adding clipboard item: {}", e);
                    }
                    last_content = content;
                }
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    async fn get_clipboard_content(&self) -> Result<String, String> {
        let result = tokio::task::spawn_blocking(|| match clipboard::ClipboardContext::new() {
            Ok(mut ctx) => ctx.get_contents().unwrap_or_default(),
            Err(_) => String::new(),
        })
        .await;

        match result {
            Ok(content) => Ok(content),
            Err(e) => Err(format!("Clipboard access error: {}", e)),
        }
    }
}
