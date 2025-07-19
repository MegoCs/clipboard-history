mod clipboard_item;
mod clipboard_manager;
mod monitor;
mod service;
mod storage;
mod ui;

use service::ClipboardService;
use ui::ConsoleInterface;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize the clipboard service (UI-agnostic core)
    let mut service = ClipboardService::new().await?;
    
    // Start clipboard monitoring and get event receiver
    let event_receiver = service.start_monitoring();
    
    // Initialize the console UI with the service
    let console_ui = ConsoleInterface::new(service, event_receiver);
    
    // Run the console interface
    console_ui.run().await
}
