use crate::service::{ClipboardService, SearchResult};
use base64::prelude::*;
use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Configuration for the popup UI
#[derive(Clone, Debug)]
pub struct PopupConfig {
    pub popup_width: f32,
    pub popup_height: f32,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            popup_width: 400.0,
            popup_height: 300.0,
        }
    }
}

/// Popup clipboard manager UI
#[derive(Clone)]
pub struct PopupClipboardUI {
    service: Arc<Mutex<ClipboardService>>,
    config: PopupConfig,

    // UI State - these will be recreated for each popup
    cursor_position: (f32, f32),
}

impl PopupClipboardUI {
    pub fn new(service: ClipboardService, config: PopupConfig) -> Self {
        Self {
            service: Arc::new(Mutex::new(service)),
            config,
            cursor_position: (0.0, 0.0),
        }
    }

    pub async fn show_popup(&mut self) -> eframe::Result<Option<usize>> {
        // Get current cursor position
        self.update_cursor_position();

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([self.config.popup_width, self.config.popup_height])
                .with_position([self.cursor_position.0, self.cursor_position.1])
                .with_decorations(true) // Enable decorations temporarily to avoid black screen
                .with_resizable(false)
                .with_transparent(false)
                .with_always_on_top()
                .with_close_button(true)
                .with_minimize_button(false)
                .with_maximize_button(false)
                .with_active(true) // Make sure the window is active and can detect focus loss
                .with_visible(true),
            ..Default::default()
        };

        let app = PopupApp::new(Arc::clone(&self.service), self.config.clone());

        println!("🪟 Starting popup window...");
        match eframe::run_native(
            "Clipboard Manager",
            native_options,
            Box::new(|_| Ok(Box::new(app))),
        ) {
            Ok(_) => {
                println!("✅ Popup closed cleanly, returning to hotkey waiting");
                // Force screen refresh to remove any shadows on Windows
                #[cfg(windows)]
                {
                    self.force_screen_refresh();
                }
                Ok(None)
            }
            Err(e) => {
                println!("❌ eframe error: {}", e);
                Err(e)
            }
        }
    }

    fn update_cursor_position(&mut self) {
        #[cfg(windows)]
        {
            use winapi::shared::windef::POINT;
            use winapi::um::winuser::GetCursorPos;
            let mut point = POINT { x: 0, y: 0 };
            unsafe {
                if GetCursorPos(&mut point) != 0 {
                    // Adjust position to ensure popup stays on screen
                    let screen_width = 1920.0; // Default screen width - could be made dynamic
                    let screen_height = 1080.0; // Default screen height - could be made dynamic

                    let mut x = point.x as f32;
                    let mut y = point.y as f32;

                    // Ensure popup doesn't go off the right edge of screen
                    if x + self.config.popup_width > screen_width {
                        x = screen_width - self.config.popup_width;
                    }

                    // Ensure popup doesn't go off the bottom edge of screen
                    if y + self.config.popup_height > screen_height {
                        y = screen_height - self.config.popup_height;
                    }

                    // Ensure popup doesn't go off the left or top edges
                    if x < 0.0 {
                        x = 0.0;
                    }
                    if y < 0.0 {
                        y = 0.0;
                    }

                    self.cursor_position = (x, y);
                } else {
                    // Fallback to center of screen if cursor position can't be retrieved
                    self.cursor_position = (100.0, 100.0);
                }
            }
        }

        #[cfg(not(windows))]
        {
            // For non-Windows platforms, use a default position
            self.cursor_position = (100.0, 100.0);
        }
    }

    #[cfg(windows)]
    fn force_screen_refresh(&self) {
        use std::ptr;
        use winapi::um::winuser::{
            RedrawWindow, RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE,
        };

        unsafe {
            // Force redraw the area where the popup was
            let rect = winapi::shared::windef::RECT {
                left: self.cursor_position.0 as i32,
                top: self.cursor_position.1 as i32,
                right: (self.cursor_position.0 + self.config.popup_width) as i32,
                bottom: (self.cursor_position.1 + self.config.popup_height) as i32,
            };

            RedrawWindow(
                ptr::null_mut(),
                &rect,
                ptr::null_mut(),
                RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN,
            );
        }
    }
}

struct PopupApp {
    service: Arc<Mutex<ClipboardService>>,
    config: PopupConfig,

    // UI State
    search_text: String,
    selected_index: usize,
    search_results: Vec<SearchResult>,
    should_close: bool,
    should_copy_selected: bool,
    selected_item_index: Option<usize>,
    data_loaded: bool,
    close_requested: bool, // Add explicit close tracking

    // Performance optimization: Cache textures to avoid recreating them
    texture_cache: std::collections::HashMap<String, egui::TextureHandle>,

    // Performance optimization: Cache style to avoid recreating every frame
    style_set: bool,
}

impl PopupApp {
    fn new(service: Arc<Mutex<ClipboardService>>, config: PopupConfig) -> Self {
        Self {
            service,
            config,
            search_text: String::new(),
            selected_index: 0,
            search_results: Vec::new(),
            should_close: false,
            should_copy_selected: false,
            selected_item_index: None,
            data_loaded: false,
            close_requested: false,
            texture_cache: std::collections::HashMap::new(),
            style_set: false,
        }
    }

    fn refresh_data(&mut self) {
        // Performance optimization: Use a more efficient approach for data loading
        let service = Arc::clone(&self.service);
        let search_text = self.search_text.clone();

        // Use a more efficient async approach with timeout to prevent hanging
        let results = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            // Add timeout to prevent hanging on slow operations
            rt.block_on(async {
                // Use a timeout for the operation
                match tokio::time::timeout(
                    std::time::Duration::from_millis(500), // 500ms timeout
                    async {
                        let service = service.lock().await;
                        if search_text.is_empty() {
                            // Show all history
                            let history = service.get_history().await;
                            history
                                .into_iter()
                                .enumerate()
                                .map(|(index, item)| SearchResult {
                                    item,
                                    index,
                                    score: None,
                                })
                                .collect::<Vec<_>>()
                        } else {
                            // Perform search with limit to improve performance
                            let (exact, fuzzy) = service.search_unified(&search_text).await;
                            let mut results = if !fuzzy.is_empty() { fuzzy } else { exact };

                            // Limit results to improve UI performance (show top 50 results)
                            results.truncate(50);
                            results
                        }
                    },
                )
                .await
                {
                    Ok(data) => data,
                    Err(_) => {
                        eprintln!("Search operation timed out");
                        Vec::new()
                    }
                }
            })
        })
        .join();

        if let Ok(data) = results {
            self.search_results = data;
            self.selected_index = 0;
            self.data_loaded = true;
        } else {
            // Fallback to empty results
            self.search_results = Vec::new();
            self.selected_index = 0;
            self.data_loaded = true;
        }
    }

    fn copy_selected_item(&mut self) {
        if self.selected_index < self.search_results.len() {
            let selected_result = &self.search_results[self.selected_index];
            self.selected_item_index = Some(selected_result.index);
            self.should_copy_selected = true;

            // Copy to clipboard in a background thread with proper error handling
            let service = Arc::clone(&self.service);
            let index = selected_result.index;
            let item_preview = selected_result.item.clean_preview(50);

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let service = service.lock().await;
                    match service.copy_to_clipboard(index).await {
                        Ok(_) => {
                            println!("✅ Item copied to clipboard!");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to copy item to clipboard: {}", e);
                            eprintln!("   Item preview: {}", item_preview);
                            eprintln!("   This may be due to corrupted image data or unsupported format.");

                            // Try to provide helpful information
                            if e.to_string().contains("Invalid buffer length") {
                                eprintln!("   Suggestion: This image may be corrupted or have invalid metadata.");
                            }
                        }
                    }
                });
            });

            // Item copied but popup stays open - no automatic closing
        }
    }
}

impl eframe::App for PopupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for window close request (built-in close button) - Cross-platform approach
        let close_requested = ctx.input(|i| i.viewport().close_requested());

        // Focus loss functionality removed - popup only closes on explicit user action

        // Alternative check for older egui versions or different platforms
        let mut additional_close_check = false;
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key: egui::Key::F4,
                    modifiers,
                    pressed: true,
                    ..
                } = event
                {
                    if modifiers.alt {
                        additional_close_check = true;
                        break;
                    }
                }
            }
        });

        if close_requested || additional_close_check {
            println!(" Window close button pressed - closing popup");

            // Prevent multiple close attempts
            if !self.should_close {
                self.should_close = true;
                self.close_requested = true;

                println!("🚪 Closing popup window...");
            }
        }

        // Prevent further processing if we're supposed to be closing
        if self.should_close || self.close_requested {
            // Just return early to avoid drawing anything else
            // The eframe loop should handle the close naturally
            return;
        }

        // Initialize data on first run
        if !self.data_loaded {
            self.refresh_data();
        }

        // Set up the popup style with bright, visible background and bigger font (only once)
        if !self.style_set {
            let mut style = (*ctx.style()).clone();
            style.visuals.window_fill = egui::Color32::WHITE; // Pure white background
            style.visuals.window_stroke =
                egui::Stroke::new(2.0, egui::Color32::from_rgb(70, 70, 70)); // Dark border for contrast
            style.visuals.panel_fill = egui::Color32::WHITE; // White panel
            style.visuals.override_text_color = Some(egui::Color32::BLACK); // Ensure text is black

            // Increase font size for better readability
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            );

            ctx.set_style(style);
            self.style_set = true;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default()
                .fill(egui::Color32::WHITE) // Pure white background
                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)))
                .rounding(egui::Rounding::same(6.0)) // Slightly rounded corners
                .inner_margin(egui::Margin::same(10.0)) // More margin for better spacing
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Search box with proper styling
                    ui.horizontal(|ui| {
                        ui.label("🔍 Search:");

                        // Style the search text box with white background and border
                        let search_style = ui.style_mut();
                        search_style.visuals.extreme_bg_color = egui::Color32::WHITE;
                        search_style.visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;
                        search_style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(248, 248, 248);
                        search_style.visuals.widgets.active.bg_fill = egui::Color32::WHITE;
                        search_style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
                        search_style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150));

                        let search_response = ui.text_edit_singleline(&mut self.search_text);

                        if search_response.changed() {
                            // Refresh search results when text changes
                            self.data_loaded = false; // Force reload
                            self.refresh_data();
                        }

                        // Auto-focus the search box when popup opens
                        search_response.request_focus();
                    });

                    ui.separator();

                    // History list with scrolling - using full available space
                    let mut should_copy = false;
                    let mut copy_index = None;

                    egui::ScrollArea::vertical()
                        .max_height(self.config.popup_height - 80.0) // Reduced space reservation for search only
                        .auto_shrink([false; 2]) // Prevent shrinking
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                        .show(ui, |ui| {
                            // Set the UI width to ensure proper scrollbar positioning
                            ui.set_min_width(self.config.popup_width - 30.0); // Leave space for scrollbar on right

                            // Display ALL search results, not just the first 10
                            for (display_index, result) in self.search_results.iter().enumerate() {
                                let is_selected = display_index == self.selected_index;

                                // Alternating background colors: white and more visible gray
                                let row_bg_color = if display_index % 2 == 0 {
                                    egui::Color32::WHITE // Even rows: white
                                } else {
                                    egui::Color32::from_rgb(230, 230, 230) // Odd rows: more visible gray
                                };

                                // Override with selection color if selected
                                let final_bg_color = if is_selected {
                                    egui::Color32::from_rgb(200, 220, 255) // Light blue selection
                                } else {
                                    row_bg_color
                                };

                                // Create a frame for the entire row with alternating background
                                let row_frame = egui::Frame::default()
                                    .fill(final_bg_color)
                                    .rounding(egui::Rounding::same(2.0))
                                    .inner_margin(egui::Margin::symmetric(8.0, 6.0));

                                // Use allocate_ui_with_layout to ensure full width background
                                let available_rect = ui.available_rect_before_wrap();
                                let item_response = ui.allocate_ui_with_layout(
                                    egui::Vec2::new(available_rect.width(), 56.0), // Increased height for bigger images
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        row_frame.show(ui, |ui| {
                                            // Ensure the frame takes full width
                                            ui.set_min_width(available_rect.width() - 16.0); // Account for margins

                                            // Check if this is an image item to display preview
                                            match &result.item.content {
                                                crate::clipboard_item::ClipboardContentType::Image { data, .. } => {
                                                    // Display image preview with text
                                                    ui.horizontal(|ui| {
                                                        // Try to decode and display the image
                                                        if let Ok(image_data) = base64::prelude::BASE64_STANDARD.decode(data) {
                                                            // Check if we have a cached texture first
                                                            let texture_id = format!("thumb_{}", &result.item.id);

                                                            if let Some(cached_texture) = self.texture_cache.get(&texture_id) {
                                                                // Use cached texture
                                                                let image = egui::Image::from_texture(cached_texture)
                                                                    .fit_to_exact_size(egui::Vec2::new(48.0, 48.0));
                                                                ui.add(image);
                                                            } else if let Ok(dynamic_image) = image::load_from_memory(&image_data) {
                                                                // Create new texture and cache it
                                                                let thumbnail_size = 48;
                                                                let thumbnail = dynamic_image.thumbnail(thumbnail_size, thumbnail_size);
                                                                let rgba_image = thumbnail.to_rgba8();
                                                                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                                                                let pixels = rgba_image.as_flat_samples();

                                                                // Create texture from image data
                                                                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                                                                let texture_handle = ui.ctx().load_texture(
                                                                    texture_id.clone(),
                                                                    color_image,
                                                                    egui::TextureOptions::default()
                                                                );

                                                                // Cache the texture for future use
                                                                self.texture_cache.insert(texture_id, texture_handle.clone());

                                                                let image = egui::Image::from_texture(&texture_handle)
                                                                    .fit_to_exact_size(egui::Vec2::new(48.0, 48.0));

                                                                ui.add(image);
                                                            } else {
                                                                // Fallback to icon if image can't be loaded
                                                                ui.label("🖼️");
                                                            }
                                                        } else {
                                                            // Fallback to icon if image can't be decoded
                                                            ui.label("🖼️");
                                                        }

                                                        // Add image info text
                                                        let item_number = display_index + 1;
                                                        ui.label(format!("{item_number}. image"));
                                                    }).response
                                                },
                                                _ => {
                                                    // Regular text-based items
                                                    ui.horizontal(|ui| {
                                                        let item_number = display_index + 1;
                                                        let preview_text = result.item.clean_preview(50);
                                                        ui.label(format!("{item_number}. {preview_text}"))
                                                    }).response
                                                }
                                            }
                                        }).response
                                    }
                                ).response;

                                // Auto-scroll to selected item when navigating with keyboard
                                if is_selected {
                                    item_response.scroll_to_me(Some(egui::Align::Center));
                                }

                                // Handle single click to select
                                if item_response.clicked() {
                                    self.selected_index = display_index;
                                }

                                // Handle double click to select and close
                                if item_response.double_clicked() {
                                    should_copy = true;
                                    copy_index = Some(display_index);
                                }

                                // Note: Removed hover selection to prevent unwanted scrolling on mouse movement
                                // Selection is now only via clicks and keyboard navigation

                                // Add separator between entries (except after the last item)
                                if display_index < self.search_results.len() - 1 {
                                    ui.separator();
                                }
                            }
                        });

                    // Handle the copy operation after the borrow ends
                    if should_copy {
                        if let Some(index) = copy_index {
                            self.selected_index = index;
                            self.copy_selected_item();
                        }
                    }
                });
            });

        // Handle keyboard input - Multiple approaches for better reliability
        let input = ctx.input(|i| i.clone());

        // Method 1: Check raw events
        for event in &input.events {
            match event {
                egui::Event::Key {
                    key: egui::Key::Escape,
                    pressed: true,
                    ..
                } => {
                    println!("🔑 ESC key pressed (raw event) - closing popup");
                    self.should_close = true;
                    self.close_requested = true;
                }
                egui::Event::Key {
                    key: egui::Key::ArrowUp,
                    pressed: true,
                    ..
                } => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                egui::Event::Key {
                    key: egui::Key::ArrowDown,
                    pressed: true,
                    ..
                } => {
                    if self.selected_index < self.search_results.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                }
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                } => {
                    if !self.search_results.is_empty()
                        && self.selected_index < self.search_results.len()
                    {
                        self.copy_selected_item();
                    }
                }
                _ => {}
            }
        }

        // Request repaint only when there's actual UI interaction (reduce CPU usage)
        let needs_repaint = !self.search_text.is_empty()
            || self.selected_index > 0
            || !self.search_results.is_empty();

        if !self.should_close && !self.close_requested && needs_repaint {
            ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60 FPS when needed
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Called when the app is being shut down
        println!("🏁 Popup app exiting");
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // Disable auto-save
        std::time::Duration::from_secs(0)
    }
}

/// Global hotkey manager for the popup
pub struct HotkeyManager {
    hotkey_id: u32,
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self { hotkey_id: 1 }
    }

    pub fn register_hotkey(&self, _hotkey: &str) -> Result<(), String> {
        // For now, we'll implement Windows-specific hotkey registration
        #[cfg(windows)]
        {
            use std::ptr;
            use winapi::um::winuser::{RegisterHotKey, MOD_CONTROL, MOD_SHIFT};

            // Parse hotkey string (for now, hardcoded to Ctrl+Shift+V)
            let modifiers = MOD_CONTROL | MOD_SHIFT;
            let key = 0x56u32; // VK_V key code

            unsafe {
                if RegisterHotKey(
                    ptr::null_mut(),
                    self.hotkey_id as i32,
                    modifiers as u32,
                    key,
                ) == 0
                {
                    return Err("Failed to register hotkey".to_string());
                }
            }
        }

        #[cfg(not(windows))]
        {
            eprintln!("Hotkey registration not implemented for this platform");
        }

        Ok(())
    }

    pub fn unregister_hotkey(&self) {
        #[cfg(windows)]
        {
            use std::ptr;
            use winapi::um::winuser::UnregisterHotKey;

            unsafe {
                UnregisterHotKey(ptr::null_mut(), self.hotkey_id as i32);
            }
        }
    }

    pub fn wait_for_hotkey(&self) -> bool {
        #[cfg(windows)]
        {
            use std::mem;
            use winapi::um::winuser::{GetMessageW, MSG, WM_HOTKEY};

            loop {
                let mut msg: MSG = unsafe { mem::zeroed() };
                let result = unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) };

                match result.cmp(&0) {
                    std::cmp::Ordering::Greater => {
                        if msg.message == WM_HOTKEY && msg.wParam == self.hotkey_id as usize {
                            return true;
                        }
                    }
                    std::cmp::Ordering::Less => break,
                    std::cmp::Ordering::Equal => {}
                }
            }
        }

        #[cfg(not(windows))]
        {
            // For non-Windows platforms, return false for now
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        false
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.unregister_hotkey();
    }
}
