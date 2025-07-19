# 📋 Clipboard History Manager

A **powerful, intelligent clipboard history manager** written in Rust with advanced search capabilities, smart content analysis, and cross-platform support.

[![CI](https://github.com/MegoCs/clipboard-history/workflows/CI/badge.svg)](https://github.com/MegoCs/clipboard-history/actions)
[![Security Audit](https://github.com/MegoCs/clipboard-history/workflows/Security%20Audit/badge.svg)](https://github.com/MegoCs/clipboard-history/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## ✨ Features

### 🚀 **Core Capabilities**
- **🖥️ Background Monitoring**: Automatically captures clipboard changes without user intervention
- **💾 Persistent Storage**: Saves clipboard history to disk and restores it on startup  
- **🔍 Intelligent Search**: Both exact text matching and fuzzy search with content-type detection
- **🎯 Smart Previews**: Context-aware content previews with size information
- **📊 Usage Analytics**: Track clipboard usage patterns and statistics
- **🌐 Cross-Platform**: Works seamlessly on Windows, macOS, and Linux

### 🧠 **Advanced Search Features**
- **Fuzzy Matching**: Find items even with typos (`"passowrd"` finds `"password"`)
- **Content Detection**: Automatically identifies JSON, URLs, code, and multi-line text
- **Smart Previews**: Shows content type and size (`JSON: {...} [1.2 KB]`)
- **Multiple Search Modes**: Exact, fuzzy, and combined search results
- **Performance Optimized**: Fast search through thousands of clipboard items

### 🛡️ **Enterprise-Grade Quality**
- **Security Audited**: Regular vulnerability scanning with `cargo audit`
- **Memory Safe**: Built with Rust for zero buffer overflows
- **Performance Optimized**: Handles content up to 10MB with smart limits
- **Test Coverage**: Comprehensive unit and integration tests
- **CI/CD Pipeline**: Automated testing and releases

> 🎬 **[See Search Demo](SEARCH_DEMO.md)** | 📖 **[CI/CD Details](CI_CD_SETUP.md)**

## 📥 Installation

### 🚀 **Quick Start (Recommended)**
Download pre-built binaries from [GitHub Releases](../../releases/latest):

```bash
# Linux
wget https://github.com/MegoCs/clipboard-history/releases/latest/download/clipboard-history-linux-x64
chmod +x clipboard-history-linux-x64
./clipboard-history-linux-x64

# macOS (Intel)
wget https://github.com/MegoCs/clipboard-history/releases/latest/download/clipboard-history-macos-x64
chmod +x clipboard-history-macos-x64
./clipboard-history-macos-x64

# macOS (Apple Silicon)
wget https://github.com/MegoCs/clipboard-history/releases/latest/download/clipboard-history-macos-arm64
chmod +x clipboard-history-macos-arm64  
./clipboard-history-macos-arm64

# Windows - Download clipboard-history-windows-x64.exe and double-click
```

### 🛠️ **Build from Source**
```bash
# Prerequisites: Rust 1.70+ (install from https://rustup.rs)
git clone https://github.com/MegoCs/clipboard-history.git
cd clipboard-history
cargo build --release

# Binary will be at: target/release/clipboard-history[.exe]
```

### 📦 **Platform Requirements**
- **Linux**: XCB libraries (`sudo apt install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`)
- **Windows**: Windows 10+ (no additional dependencies)
- **macOS**: macOS 10.15+ (both Intel and Apple Silicon supported)

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
- **Type 's' or 'search'**: Enter interactive search mode
- **Type 'c' or 'clear'**: Clear all clipboard history
- **Type 'q' or 'quit'**: Return to main menu

### Interactive Search Mode

The search system provides two types of matching:

**Fuzzy Search** (default):
- Finds items even with typos or partial matches
- Results are ranked by relevance score
- Example: searching "hello" might find "Hello World!" or "helo there"

**Text Search** (fallback):
- Exact substring matching (case-insensitive)
- Reliable for precise searches

**Search Commands**:
- **Enter search term**: Find matching clipboard items
- **Type a number**: Copy that search result to clipboard
- **Type 'h' or 'help'**: Show search help
- **Type 'q' or 'quit'**: Exit search mode

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
7. **Enter search terms** like "password" or "email" to find specific items
8. **Select numbered results** to copy them back to clipboard
9. **Type 'q'** to return to main menu

## Features in Detail

### Background Monitoring
The app continuously monitors your clipboard every 500ms and automatically saves any new text you copy.

### Persistent Storage
Your clipboard history is saved to:
- **Windows**: `%APPDATA%\clipboard-history\history.json`
- **macOS**: `~/Library/Application Support/clipboard-history/history.json`
- **Linux**: `~/.local/share/clipboard-history/history.json`

### Smart Search
The search function offers two modes:
1. **Fuzzy matching** - finds items even with typos or partial matches, ranked by relevance
2. **Text matching** - exact substring search for precise queries

Search supports:
- Case-insensitive matching
- Partial word matching
- Typo tolerance (fuzzy search)
- Relevance scoring and ranking

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
- Enter → Search for items
- 1-15 → Copy search result
- h → Search help
- q → Exit search

## License

Open source project. Feel free to modify and distribute as needed.
