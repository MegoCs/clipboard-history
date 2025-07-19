use crate::clipboard_manager::ClipboardManager;
use std::io;
use std::sync::Arc;

pub struct UserInterface {
    manager: Arc<ClipboardManager>,
}

impl UserInterface {
    pub fn new(manager: Arc<ClipboardManager>) -> Self {
        Self { manager }
    }

    pub async fn run(&self) -> io::Result<()> {
        println!("Clipboard Manager Started!");
        println!("Items loaded: {}", self.manager.get_history_count().await);
        println!("Storage location: {:?}", self.manager.get_storage_path());

        loop {
            self.show_menu().await;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let command = input.trim().to_lowercase();
            
            match command.as_str() {
                "q" | "quit" | "exit" => {
                    println!("Goodbye!");
                    break;
                }
                "h" | "help" => {
                    self.show_help();
                }
                "c" | "clear" => {
                    self.clear_history().await?;
                }
                "s" | "search" => {
                    self.search_interactive().await?;
                }
                "" => {
                    self.show_history().await;
                }
                _ => {
                    // Try to parse as number for item selection
                    if let Ok(num) = command.parse::<usize>() {
                        self.select_item(num).await;
                    } else {
                        println!("Unknown command: '{}'. Type 'h' for help.", command);
                    }
                }
            }
        }

        Ok(())
    }

    async fn show_menu(&self) {
        println!("\n=== Clipboard Manager ===");
        println!("Press Enter to view history, or type a command:");
        print!("> ");
    }

    fn show_help(&self) {
        println!("\n=== Commands ===");
        println!("  [Enter]     - View clipboard history");
        println!("  [number]    - Select item by number (future feature)");
        println!("  h, help     - Show this help");
        println!("  s, search   - Search clipboard history");
        println!("  c, clear    - Clear all history");
        println!("  q, quit     - Exit the program");
    }

    async fn show_history(&self) {
        let history = self.manager.get_history().await;
        
        if history.is_empty() {
            println!("No items yet. Try copying something!");
            return;
        }

        println!("\n=== Clipboard History ({} items) ===", history.len());
        for (i, item) in history.iter().enumerate().take(20) {
            let preview = item.preview(80);
            let timestamp = item.formatted_timestamp();
            println!("{}. {} [{}]", i + 1, preview, timestamp);
        }

        if history.len() > 20 {
            println!("... and {} more items", history.len() - 20);
        }
    }

    async fn search_interactive(&self) -> io::Result<()> {
        println!("Enter search term:");
        let mut query = String::new();
        io::stdin().read_line(&mut query)?;
        
        let query = query.trim();
        if query.is_empty() {
            println!("Empty search query.");
            return Ok(());
        }

        let results = self.manager.search_history(query).await;
        
        if results.is_empty() {
            println!("No items found matching '{}'", query);
        } else {
            println!("\n=== Search Results for '{}' ({} found) ===", query, results.len());
            for (original_index, item) in results.iter().take(10) {
                let preview = item.preview(80);
                let timestamp = item.formatted_timestamp();
                println!("{}. {} [{}]", original_index + 1, preview, timestamp);
            }
            
            if results.len() > 10 {
                println!("... and {} more results", results.len() - 10);
            }
        }

        Ok(())
    }

    async fn clear_history(&self) -> io::Result<()> {
        println!("Are you sure you want to clear all clipboard history? (y/N):");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            self.manager.clear_history().await?;
            println!("Clipboard history cleared.");
        } else {
            println!("Clear cancelled.");
        }

        Ok(())
    }

    async fn select_item(&self, number: usize) {
        if number == 0 {
            println!("Item numbers start from 1.");
            return;
        }

        if let Some(item) = self.manager.get_item_by_index(number - 1).await {
            println!("Selected item {}:", number);
            println!("Content: {}", item.content);
            println!("Timestamp: {}", item.formatted_timestamp());
            println!("(Copy-to-clipboard feature coming soon!)");
        } else {
            println!("Item {} not found.", number);
        }
    }
}
