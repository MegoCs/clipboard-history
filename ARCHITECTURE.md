# Clipboard Manager - Architecture Documentation

## Project Structure Overview

The clipboard manager has been refactored into a clean, modular architecture with separation of concerns:

```
src/
├── main.rs              # Entry point and orchestration
├── clipboard_item.rs    # ClipboardItem data structure
├── clipboard_manager.rs # Core business logic
├── monitor.rs           # Background clipboard monitoring
├── storage.rs           # File persistence layer
└── ui.rs               # User interface and commands
```

## Module Responsibilities

### `main.rs` - Application Entry Point
- Initializes all components
- Orchestrates startup sequence
- Manages async task lifecycle
- Handles graceful shutdown

### `clipboard_item.rs` - Data Model
- **ClipboardItem struct**: Core data structure for clipboard entries
- **Fields**: content (String), timestamp (u64), id (usize)
- **Methods**:
  - `new()`: Constructor with auto-generated timestamp
  - `preview()`: Truncated content preview
  - `formatted_timestamp()`: Human-readable timestamp

### `clipboard_manager.rs` - Business Logic
- **ClipboardManager struct**: Central coordinator for all clipboard operations
- **Key Features**:
  - Thread-safe history management with `Arc<Mutex<VecDeque<ClipboardItem>>>`
  - Duplicate detection and prevention
  - History size limiting (1000 items max)
  - Search functionality
  - CRUD operations for clipboard items
- **Public API**:
  - `add_item()`: Add new clipboard content
  - `get_history()`: Retrieve all items
  - `search_history()`: Find items by exact text matching
  - `fuzzy_search_history()`: Find items with fuzzy matching and relevance scoring
  - `copy_item_to_clipboard()`: Copy historical item back to system clipboard
  - `clear_history()`: Remove all items
  - `get_item_by_index()`: Get specific item

### `storage.rs` - Persistence Layer
- **Storage struct**: Handles file I/O operations
- **Responsibilities**:
  - JSON serialization/deserialization
  - File path management (OS-specific directories)
  - Directory creation and validation
  - Error handling for file operations
- **Key Methods**:
  - `new()`: Initialize storage with proper directory structure
  - `load_history()`: Read existing history from disk
  - `save_history()`: Write current history to disk

### `monitor.rs` - Background Monitoring
- **ClipboardMonitor struct**: Handles clipboard change detection
- **Features**:
  - Configurable polling interval (default: 500ms)
  - Async clipboard access with proper error handling
  - Content change detection and duplicate filtering
  - Integration with ClipboardManager for automatic item addition
- **Architecture**:
  - Uses `tokio::task::spawn_blocking` for clipboard API calls
  - Runs continuously in background task
  - Thread-safe communication with main application

### `ui.rs` - User Interface
- **UserInterface struct**: Manages all user interactions
- **Command System**:
  - Menu-driven interface with clear prompts
  - Multiple command types (view, search, clear, help, quit)
  - Input validation and error handling
  - Contextual help and feedback
- **Key Features**:
  - Interactive search with both fuzzy and text matching
  - Search loop mode for multiple queries without restart
  - Paginated history display (20 items max)
  - Item selection by number with instant clipboard copy
  - Confirmation dialogs for destructive actions
  - Human-readable timestamps and content previews

## Data Flow Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   System        │───▶│  ClipboardMonitor │───▶│ ClipboardManager│
│   Clipboard     │    │  (Background)     │    │  (Business      │
└─────────────────┘    └──────────────────┘    │   Logic)        │
                                               └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   User          │◄───│  UserInterface   │◄───│    Storage      │
│   Terminal      │    │  (Presentation)   │    │ (Persistence)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Threading Model

- **Main Thread**: Handles user interface and user input
- **Background Task**: Monitors clipboard changes continuously
- **Blocking Tasks**: Clipboard API calls (wrapped in `spawn_blocking`)
- **Synchronization**: `Arc<Mutex<>>` for thread-safe data sharing
- **Task Management**: Proper cleanup on application exit

## Error Handling Strategy

- **Storage Errors**: Graceful fallback to empty history on load failures
- **Clipboard Errors**: Silent recovery with error logging
- **UI Errors**: User-friendly error messages with retry options
- **Thread Errors**: Proper task cancellation and resource cleanup

## Performance Considerations

- **Memory Management**: Fixed-size circular buffer (1000 items max)
- **I/O Optimization**: Async file operations with proper buffering
- **CPU Efficiency**: Minimal polling overhead (500ms intervals)
- **Resource Cleanup**: Proper Arc/Mutex lifecycle management

## Extension Points

The modular architecture supports easy extension:

1. **New Storage Backends**: Implement additional storage providers
2. **Enhanced UI**: Add terminal UI libraries or web interface
3. **Advanced Search**: ✅ Implemented fuzzy matching and dual search modes
4. **Clipboard Copy-back**: ✅ Implemented item copying back to system clipboard
5. **Sync Features**: Add cloud synchronization capabilities
6. **Plugins**: Support for custom content processors or filters

## Dependencies Management

- **Core Dependencies**: Minimal, focused on essential functionality
- **Feature Flags**: Potential for optional feature compilation
- **Version Pinning**: Stable dependencies for reliable builds
- **Platform Support**: Cross-platform compatibility maintained
