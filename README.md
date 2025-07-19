# Clipboard History Manager

A powerful, searchable clipboard history manager written in Rust that runs in the background and keeps track of all your copied items.

## Features

- **Background Monitoring**: Automatically captures clipboard changes without user intervention
- **Persistent Storage**: Saves clipboard history to disk and restores it on startup  
- **Fuzzy Search**: Fast, intelligent search through your clipboard history
- **Simple Interface**: Clean, number-based interface for easy selection
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Lightweight**: Minimal resource usage while running in the background

## Installation

1. Make sure you have Rust installed (https://rustup.rs/)
2. Clone or download this project
3. Build the application:
   ```bash
   cargo build --release
   ```

## Usage

### Starting the Application
```bash
cargo run
```

The application will:
1. Start monitoring your clipboard in the background
2. Display a welcome message and wait for your commands

### Main Menu Commands

- **Press Enter**: Open clipboard history viewer
- **Type 'exit'**: Quit the application
- **Ctrl+C**: Force quit

### History Viewer Interface

When viewing clipboard history:
- **Type a number (1-20)**: Copy that item to clipboard and return to main menu
- **Type 's' or 'search'**: Enter search mode
- **Type 'c' or 'clear'**: Clear all clipboard history
- **Type 'q' or 'quit'**: Return to main menu

### Search Mode

When searching:
- **Enter search term**: Find items containing your search text (fuzzy matching)
- **Type a number**: Copy that search result to clipboard
- **Type 'q' or 'quit'**: Exit search mode
- **Type 'b' or 'back'**: Return to main history view

## Example Usage

1. **Start the app**: `cargo run`
2. **Copy some text** in any application (Ctrl+C)
3. **Press Enter** in the clipboard manager
4. **See your history** with timestamps and previews
5. **Type '1'** to copy the first item back to clipboard
6. **Type 's'** to search through your history
7. **Type 'q'** to return to main menu

## Features in Detail

### Background Monitoring
The app continuously monitors your clipboard every 500ms and automatically saves any new text you copy.

### Persistent Storage
Your clipboard history is saved to:
- **Windows**: `%APPDATA%\clipboard-history\history.json`
- **macOS**: `~/Library/Application Support/clipboard-history/history.json`
- **Linux**: `~/.local/share/clipboard-history/history.json`

### Smart Search
The search function uses fuzzy matching, so you can find items even with partial matches or typos.

## Configuration

The application stores up to 1,000 clipboard items by default. You can modify this limit by changing the `max_items` value in the `ClipboardManager::new()` function in `src/main.rs`.

## Building for Production

To create an optimized executable:
```bash
cargo build --release
```

The executable will be located at:
- Windows: `target/release/clipboard-history.exe`
- Unix: `target/release/clipboard-history`

## Running Automatically

### Windows
Create a batch file `start-clipboard-manager.bat`:
```batch
@echo off
cd /d "C:\path\to\clipboard-history"
cargo run --release
```

### Linux/macOS
Create a shell script:
```bash
#!/bin/bash
cd /path/to/clipboard-history
cargo run --release
```

## Dependencies

- **clipboard**: Cross-platform clipboard access
- **tokio**: Async runtime for background processing
- **serde & serde_json**: Data serialization for persistence
- **chrono**: Date and time handling for timestamps
- **fuzzy-matcher**: Intelligent search functionality
- **dirs**: OS-specific directory paths

## Troubleshooting

### "No clipboard history found"
This means you haven't copied any text since starting the application. Copy some text (Ctrl+C) and try again.

### Permission Issues
On some systems, clipboard access may require additional permissions. Make sure your terminal has clipboard access rights.

### Performance
The app checks clipboard every 500ms. For better performance on slower systems, increase the interval in the `clipboard_monitor` function.

## Commands Quick Reference

**Main Menu:**
- Enter → View history
- exit → Quit app

**History Viewer:**
- 1-20 → Copy item number
- s → Search
- c → Clear history  
- q → Back to main

**Search Mode:**
- 1-20 → Copy search result
- q → Exit search
- b → Back to history

## License

Open source project. Feel free to modify and distribute as needed.
