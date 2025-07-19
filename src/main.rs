mod clipboard_item;
mod clipboard_manager;
mod monitor;
mod storage;
mod ui;

use clipboard_manager::ClipboardManager;
use monitor::ClipboardMonitor;
use ui::UserInterface;
use std::io;
use std::sync::Arc;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize the clipboard manager
    let manager = Arc::new(ClipboardManager::new().await?);
    
    // Start clipboard monitoring in background
    let monitor = ClipboardMonitor::new(Arc::clone(&manager));
    let monitor_task = tokio::spawn(async move {
        monitor.start_monitoring().await;
    });
    
    // Start the user interface
    let ui = UserInterface::new(manager);
    let ui_result = ui.run().await;
    
    // Cancel the monitoring task when UI exits
    monitor_task.abort();
    
    ui_result
}
