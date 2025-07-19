use crate::clipboard_manager::ClipboardManager;
use crate::clipboard_item::ClipboardItem;
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
        println!("  [number]    - Select and copy item by number to clipboard");
        println!("  h, help     - Show this help");
        println!("  s, search   - Interactive search through clipboard history");
        println!("                (supports both exact text and fuzzy matching)");
        println!("  c, clear    - Clear all history (with confirmation)");
        println!("  q, quit     - Exit the program");
        println!("\nSearch Mode:");
        println!("  - Enter search terms to find matching clipboard items");
        println!("  - Fuzzy matching finds items even with typos or partial matches");
        println!("  - Select numbered results to copy them back to clipboard");
        println!("  - Type 'q' in search to return to main menu");
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
        loop {
            println!("\n=== Search Mode ===");
            println!("Enter search term (or 'q' to quit, 'h' for help):");
            print!("> ");
            
            let mut query = String::new();
            io::stdin().read_line(&mut query)?;
            
            let query = query.trim();
            
            match query.to_lowercase().as_str() {
                "q" | "quit" | "exit" => {
                    println!("Exiting search mode.");
                    break;
                }
                "h" | "help" => {
                    self.show_search_help();
                    continue;
                }
                "" => {
                    println!("Empty search query. Please enter a search term.");
                    continue;
                }
                _ => {
                    self.perform_search(query).await?;
                }
            }
        }
        Ok(())
    }

    fn show_search_help(&self) {
        println!("\n=== Search Help ===");
        println!("Commands:");
        println!("  - Enter any text to search clipboard history");
        println!("  - Search supports both exact text matching and fuzzy matching");
        println!("  - After search results, type a number to select and copy an item");
        println!("  - Type 'f' to toggle fuzzy search mode");
        println!("  - Type 'q' to quit search mode");
        println!("  - Type 'h' for this help message");
    }

    async fn perform_search(&self, query: &str) -> io::Result<()> {
        // First try fuzzy search for better results
        let fuzzy_results = self.manager.fuzzy_search_history(query).await;
        let text_results = self.manager.search_history(query).await;
        
        if fuzzy_results.is_empty() && text_results.is_empty() {
            println!("No items found matching '{}'", query);
            return Ok(());
        }

        // Display fuzzy results first (they're usually more relevant)
        if !fuzzy_results.is_empty() {
            println!("\n=== Fuzzy Search Results for '{}' ({} found) ===", query, fuzzy_results.len());
            self.display_search_results_with_scores(&fuzzy_results).await?;
        } else if !text_results.is_empty() {
            println!("\n=== Search Results for '{}' ({} found) ===", query, text_results.len());
            self.display_search_results(&text_results).await?;
        }

        Ok(())
    }

    async fn display_search_results_with_scores(&self, results: &[(usize, ClipboardItem, i64)]) -> io::Result<()> {
        let display_count = results.len().min(15);
        
        for (display_num, (_original_index, item, score)) in results.iter().take(display_count).enumerate() {
            let preview = item.preview(70);
            let timestamp = item.formatted_timestamp();
            println!("{}. [Score: {}] {} [{}]", 
                display_num + 1, score, preview, timestamp);
        }
        
        if results.len() > display_count {
            println!("... and {} more results (showing top {})", 
                results.len() - display_count, display_count);
        }

        self.handle_search_selection(results.iter().map(|(idx, item, _)| (*idx, item.clone())).collect()).await
    }

    async fn display_search_results(&self, results: &[(usize, ClipboardItem)]) -> io::Result<()> {
        let display_count = results.len().min(15);
        
        for (display_num, (_original_index, item)) in results.iter().take(display_count).enumerate() {
            let preview = item.preview(80);
            let timestamp = item.formatted_timestamp();
            println!("{}. {} [{}]", display_num + 1, preview, timestamp);
        }
        
        if results.len() > display_count {
            println!("... and {} more results (showing top {})", 
                results.len() - display_count, display_count);
        }

        self.handle_search_selection(results.to_vec()).await
    }

    async fn handle_search_selection(&self, results: Vec<(usize, ClipboardItem)>) -> io::Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        println!("\nActions:");
        println!("- Type a number (1-{}) to copy that item to clipboard", results.len().min(15));
        println!("- Press Enter to continue searching");
        println!("- Type 'q' to quit search");
        print!("> ");

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "" => return Ok(()), // Continue searching
            "q" | "quit" => return Ok(()),
            _ => {
                if let Ok(num) = input.parse::<usize>() {
                    if num > 0 && num <= results.len().min(15) {
                        let (original_index, item) = &results[num - 1];
                        
                        println!("\nSelected item {}:", num);
                        println!("Content: {}", item.content);
                        println!("Timestamp: {}", item.formatted_timestamp());
                        
                        // Copy to clipboard
                        match self.manager.copy_item_to_clipboard(*original_index).await {
                            Ok(true) => {
                                println!("✅ Successfully copied to clipboard!");
                            }
                            Ok(false) => {
                                println!("❌ Failed to copy to clipboard.");
                            }
                            Err(e) => {
                                println!("❌ Error copying to clipboard: {}", e);
                            }
                        }
                    } else {
                        println!("Invalid selection. Please choose a number between 1 and {}.", results.len().min(15));
                    }
                } else {
                    println!("Invalid input. Please enter a number or 'q' to quit.");
                }
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
            println!("\nSelected item {}:", number);
            println!("Content: {}", item.content);
            println!("Timestamp: {}", item.formatted_timestamp());
            
            // Copy to clipboard
            match self.manager.copy_item_to_clipboard(number - 1).await {
                Ok(true) => {
                    println!("✅ Successfully copied to clipboard!");
                }
                Ok(false) => {
                    println!("❌ Failed to copy to clipboard.");
                }
                Err(e) => {
                    println!("❌ Error copying to clipboard: {}", e);
                }
            }
        } else {
            println!("Item {} not found.", number);
        }
    }
}
