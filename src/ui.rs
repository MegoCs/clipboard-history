use crate::monitor::ClipboardEvent;
use crate::service::{ClipboardService, SearchResult};
use std::io;
use tokio::sync::broadcast;

/// Console-based user interface
/// This can be easily replaced with a desktop GUI, web interface, etc.
pub struct ConsoleInterface {
    service: ClipboardService,
    event_receiver: Option<broadcast::Receiver<ClipboardEvent>>,
}

impl ConsoleInterface {
    pub fn new(
        service: ClipboardService, 
        event_receiver: Option<broadcast::Receiver<ClipboardEvent>>
    ) -> Self {
        Self { service, event_receiver }
    }

    pub async fn run(mut self) -> io::Result<()> {
        // Display startup information
        self.show_startup_info().await;
        
        // Start listening for clipboard events in the background
        if let Some(mut receiver) = self.event_receiver.take() {
            tokio::spawn(async move {
                while let Ok(event) = receiver.recv().await {
                    Self::handle_clipboard_event(event);
                }
            });
        }

        // Main UI loop
        self.main_loop().await
    }

    async fn show_startup_info(&self) {
        let count = self.service.get_history_count().await;
        let storage_path = self.service.get_storage_path();
        
        println!("Clipboard Manager Started!");
        println!("Items loaded: {}", count);
        println!("Storage location: {:?}", storage_path);
    }

    fn handle_clipboard_event(event: ClipboardEvent) {
        match event {
            ClipboardEvent::ItemAdded { preview } => {
                println!("New clipboard: {:?}", preview);
            }
            ClipboardEvent::Error { message } => {
                eprintln!("Clipboard error: {}", message);
            }
            ClipboardEvent::Started => {
                println!("Clipboard monitoring started");
            }
        }
    }

    async fn main_loop(&self) -> io::Result<()> {
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
        let history = self.service.get_history().await;

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

    async fn perform_search(&self, query: &str) -> io::Result<()> {
        let (exact_results, fuzzy_results) = self.service.search_unified(query).await;

        if exact_results.is_empty() && fuzzy_results.is_empty() {
            println!("No items found matching '{}'", query);
            return Ok(());
        }

        // Display fuzzy results first (they're usually more relevant)
        if !fuzzy_results.is_empty() {
            println!(
                "\n=== Fuzzy Search Results for '{}' ({} found) ===",
                query,
                fuzzy_results.len()
            );
            self.display_search_results_with_scores(&fuzzy_results).await?;
        } else if !exact_results.is_empty() {
            println!(
                "\n=== Search Results for '{}' ({} found) ===",
                query,
                exact_results.len()
            );
            self.display_search_results_exact(&exact_results).await?;
        }

        Ok(())
    }

    fn show_search_help(&self) {
        println!("\n=== Search Help ===");
        println!("Commands:");
        println!("  - Enter any text to search clipboard history");
        println!("  - Search supports both exact text matching and fuzzy matching");
        println!("  - After search results, type a number to select and copy an item");
        println!("  - Type 'q' to quit search mode");
        println!("  - Type 'h' for this help message");
    }

    async fn display_search_results_with_scores(
        &self,
        results: &[SearchResult],
    ) -> io::Result<()> {
        let display_count = results.len().min(15);

        for (display_num, result) in results.iter().take(display_count).enumerate() {
            let preview = result.item.preview(70);
            let timestamp = result.item.formatted_timestamp();
            let score = result.score.unwrap_or(0);
            println!(
                "{}. [Score: {}] {} [{}]",
                display_num + 1,
                score,
                preview,
                timestamp
            );
        }

        if results.len() > display_count {
            println!(
                "... and {} more results (showing top {})",
                results.len() - display_count,
                display_count
            );
        }

        self.handle_search_selection_with_scores(results).await
    }

    async fn display_search_results_exact(&self, results: &[SearchResult]) -> io::Result<()> {
        let display_count = results.len().min(15);

        for (display_num, result) in results.iter().take(display_count).enumerate() {
            let preview = result.item.preview(80);
            let timestamp = result.item.formatted_timestamp();
            println!("{}. {} [{}]", display_num + 1, preview, timestamp);
        }

        if results.len() > display_count {
            println!(
                "... and {} more results (showing top {})",
                results.len() - display_count,
                display_count
            );
        }

        self.handle_search_selection_exact_search_result(results).await
    }

    async fn handle_search_selection_with_scores(
        &self,
        results: &[SearchResult],
    ) -> io::Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        println!("\nActions:");
        println!(
            "- Type a number (1-{}) to copy that item to clipboard",
            results.len().min(15)
        );
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
                        let result = &results[num - 1];

                        println!("\nSelected item {}:", num);
                        println!("Content: {}", result.item.content);
                        println!("Timestamp: {}", result.item.formatted_timestamp());

                        // Copy to clipboard using the search result's index, not the item ID
                        match self.service.copy_to_clipboard(result.index).await {
                            Ok(true) => {
                                println!("✅ Successfully copied to clipboard!");
                            }
                            Ok(false) => {
                                println!("❌ Failed to copy to clipboard.");
                            }
                            Err(e) => {
                                println!("❌ Error copying to clipboard: {:?}", e);
                            }
                        }
                    } else {
                        println!(
                            "Invalid selection. Please choose a number between 1 and {}.",
                            results.len().min(15)
                        );
                    }
                } else {
                    println!("Invalid input. Please enter a number or 'q' to quit.");
                }
            }
        }

        Ok(())
    }

    async fn handle_search_selection_exact_search_result(
        &self,
        results: &[SearchResult],
    ) -> io::Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        println!("\nActions:");
        println!(
            "- Type a number (1-{}) to copy that item to clipboard",
            results.len().min(15)
        );
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
                        let result = &results[num - 1];

                        println!("\nSelected item {}:", num);
                        println!("Content: {}", result.item.content);
                        println!("Timestamp: {}", result.item.formatted_timestamp());

                        // Copy to clipboard using the search result's index, not the item ID
                        match self.service.copy_to_clipboard(result.index).await {
                            Ok(true) => {
                                println!("✅ Successfully copied to clipboard!");
                            }
                            Ok(false) => {
                                println!("❌ Failed to copy to clipboard.");
                            }
                            Err(e) => {
                                println!("❌ Error copying to clipboard: {:?}", e);
                            }
                        }
                    } else {
                        println!(
                            "Invalid selection. Please choose a number between 1 and {}.",
                            results.len().min(15)
                        );
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
            if let Err(e) = self.service.clear_history().await {
                println!("❌ Error clearing history: {:?}", e);
            } else {
                println!("Clipboard history cleared.");
            }
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

        let history = self.service.get_history().await;
        if let Some(item) = history.get(number - 1) {
            println!("\nSelected item {}:", number);
            println!("Content: {}", item.content);
            println!("Timestamp: {}", item.formatted_timestamp());

            // Copy to clipboard using the array index (number - 1)
            match self.service.copy_to_clipboard(number - 1).await {
                Ok(true) => {
                    println!("✅ Successfully copied to clipboard!");
                }
                Ok(false) => {
                    println!("❌ Failed to copy to clipboard.");
                }
                Err(e) => {
                    println!("❌ Error copying to clipboard: {:?}", e);
                }
            }
        } else {
            println!("Item {} not found.", number);
        }
    }
}
