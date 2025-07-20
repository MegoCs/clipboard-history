mod clipboard_item;
mod clipboard_manager;
mod monitor;
mod popup_ui;
mod service;
mod storage;

use popup_ui::{HotkeyManager, PopupClipboardUI, PopupConfig};
use service::ClipboardService;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    run_popup_mode().await
}

async fn run_popup_mode() -> io::Result<()> {
    println!("Starting clipboard manager...");
    println!("Press Ctrl+Shift+V to open clipboard popup");

    // Initialize the clipboard service
    let mut service = ClipboardService::new().await?;

    // Start clipboard monitoring
    let _event_receiver = service.start_monitoring();

    // Set up hotkey manager
    let hotkey_manager = HotkeyManager::new();
    if let Err(e) = hotkey_manager.register_hotkey("Ctrl+Shift+V") {
        eprintln!("Failed to register hotkey: {}", e);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Hotkey registration failed: {e}"),
        ));
    }

    println!("Hotkey registered successfully. Waiting for Ctrl+Shift+V...");

    // Main loop: wait for hotkey, show popup
    loop {
        if hotkey_manager.wait_for_hotkey() {
            println!("Hotkey pressed! Opening popup...");

            // Create popup UI
            let config = PopupConfig::default();
            let mut popup_ui = PopupClipboardUI::new(service.clone(), config);

            // Show the popup and handle the result
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(popup_ui.show_popup())
                })
            }));

            match result {
                Ok(Ok(_selected_index)) => {
                    println!("✅ Popup window closed successfully");
                }
                Ok(Err(e)) => {
                    eprintln!("❌ Error showing popup: {}", e);
                }
                Err(_) => {
                    println!("⚠️ Popup exited unexpectedly, but continuing...");
                }
            }

            println!("Popup closed. Waiting for next hotkey press...");
        }
    }
}
