//! Playlist data model and CRUD operations.
//!
//! A playlist is an ordered collection of widget items that rotate at a fixed interval.
//! Unlike schedules (which trigger at specific times), playlists cycle continuously.

use std::path::Path;

use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli_display::{print_error, print_progress, print_success};
use crate::errors::VestaboardError;
use crate::scheduler::{CUSTOM_ALPHABET, ID_LENGTH};

/// Generate a unique ID for a playlist item (same format as schedule tasks)
fn generate_item_id() -> String {
    nanoid!(ID_LENGTH, CUSTOM_ALPHABET)
}

/// A single item in a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    /// Unique identifier (auto-generated if not provided)
    #[serde(default = "generate_item_id")]
    pub id: String,
    /// Widget type to execute (e.g., "weather", "text", "sat-word")
    pub widget: String,
    /// Widget-specific input (varies by widget type)
    pub input: Value,
}

impl PlaylistItem {
    /// Create a new playlist item with auto-generated ID
    ///
    /// Note: Widget type is validated at execution time via execute_widget(),
    /// matching the pattern used by ScheduledTask.
    pub fn new(widget: String, input: Value) -> Self {
        Self {
            id: generate_item_id(),
            widget,
            input,
        }
    }
}

/// Default interval between playlist item rotations (5 minutes)
fn default_interval() -> u64 {
    300
}

/// A playlist of widget items that rotate at a fixed interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Seconds between item rotations (minimum: 60, default: 300)
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    /// Ordered list of items to rotate through
    #[serde(default)]
    pub items: Vec<PlaylistItem>,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            interval_seconds: default_interval(),
            items: Vec::new(),
        }
    }
}

impl Playlist {
    /// Add an item to the playlist
    pub fn add_item(&mut self, item: PlaylistItem) {
        self.items.push(item);
    }

    /// Add an item by widget name and return the generated ID
    ///
    /// Note: Widget type is validated at execution time via execute_widget(),
    /// matching the pattern used by Schedule.
    pub fn add_widget(&mut self, widget: &str, input: Value) -> String {
        let item = PlaylistItem::new(widget.to_string(), input);
        let id = item.id.clone();
        self.items.push(item);
        id
    }

    /// Remove an item by ID, returns true if item was found and removed
    pub fn remove_item(&mut self, id: &str) -> bool {
        let len_before = self.items.len();
        self.items.retain(|item| item.id != id);
        self.items.len() < len_before
    }

    /// Clear all items from the playlist
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Check if the playlist has no items
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of items in the playlist
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Get an item by ID
    pub fn get_item(&self, id: &str) -> Option<&PlaylistItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Get an item by index
    pub fn get_item_by_index(&self, index: usize) -> Option<&PlaylistItem> {
        self.items.get(index)
    }

    /// Find the index of an item by ID
    pub fn find_index_by_id(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }

    /// Validate that the interval is at least 60 seconds
    pub fn validate_interval(&self) -> Result<(), VestaboardError> {
        if self.interval_seconds < 60 {
            return Err(VestaboardError::validation_error(
                "Playlist interval must be at least 60 seconds",
            ));
        }
        Ok(())
    }

    // --- File operations (following scheduler.rs pattern) ---

    /// Load playlist from file (with progress messages)
    pub fn load(path: &Path) -> Result<Self, VestaboardError> {
        Self::load_internal(path, false)
    }

    /// Load playlist without printing progress (for internal operations)
    pub fn load_silent(path: &Path) -> Result<Self, VestaboardError> {
        Self::load_internal(path, true)
    }

    fn load_internal(path: &Path, silent: bool) -> Result<Self, VestaboardError> {
        log::debug!("Loading playlist from {}", path.display());

        match std::fs::read_to_string(path) {
            Ok(content) if content.trim().is_empty() => {
                log::info!("Playlist file is empty, using defaults");
                Ok(Self::default())
            }
            Ok(content) => match serde_json::from_str::<Self>(&content) {
                Ok(playlist) => {
                    log::info!("Loaded playlist with {} items", playlist.items.len());
                    Ok(playlist)
                }
                Err(e) => {
                    log::error!("Failed to parse playlist: {}", e);
                    let error =
                        VestaboardError::json_error(e, &format!("parsing playlist from {}", path.display()));
                    if !silent {
                        print_error(&error.to_user_message());
                    }
                    Err(error)
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!("Playlist file not found, using default");
                let playlist = Self::default();
                // Don't auto-save on first access to avoid creating files in unexpected locations
                Ok(playlist)
            }
            Err(e) => {
                log::error!("Failed to read playlist file: {}", e);
                let error =
                    VestaboardError::io_error(e, &format!("reading playlist from {}", path.display()));
                if !silent {
                    print_error(&error.to_user_message());
                }
                Err(error)
            }
        }
    }

    /// Save playlist to file (with progress messages)
    pub fn save(&self, path: &Path) -> Result<(), VestaboardError> {
        self.save_internal(path, false)
    }

    /// Save playlist without printing progress (for internal operations)
    pub fn save_silent(&self, path: &Path) -> Result<(), VestaboardError> {
        self.save_internal(path, true)
    }

    fn save_internal(&self, path: &Path, silent: bool) -> Result<(), VestaboardError> {
        log::debug!(
            "Saving playlist with {} items to {}",
            self.items.len(),
            path.display()
        );

        if !silent {
            print_progress("Saving playlist...");
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                let error =
                    VestaboardError::io_error(e, &format!("creating directory for {}", path.display()));
                if !silent {
                    print_error(&error.to_user_message());
                }
                return Err(error);
            }
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| VestaboardError::json_error(e, "serializing playlist"))?;

        match std::fs::write(path, content) {
            Ok(_) => {
                log::info!("Playlist saved to {}", path.display());
                if !silent {
                    print_success("Playlist saved.");
                }
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to save playlist: {}", e);
                let error =
                    VestaboardError::io_error(e, &format!("saving playlist to {}", path.display()));
                if !silent {
                    print_error(&error.to_user_message());
                }
                Err(error)
            }
        }
    }
}

// --- CLI functions (following scheduler.rs pattern) ---

use std::time::Duration;

use crate::api_broker::{handle_message, MessageDestination};
use crate::config::Config;
use crate::process_control::ProcessController;
use crate::runner::keyboard::{InputSource, KeyboardListener};
use crate::runner::lock::InstanceLock;
use crate::runner::playlist_runner::PlaylistRunner;
use crate::runner::{ControlFlow, Runner};
use crate::widgets::resolver::execute_widget;
use crate::widgets::widget_utils::error_to_display_message;

/// Get the default playlist file path from config
fn get_playlist_path() -> std::path::PathBuf {
    Config::load_silent()
        .map(|c| c.get_playlist_file_path())
        .unwrap_or_else(|_| std::path::PathBuf::from("data/playlist.json"))
}

/// Add an item to the playlist and save
pub fn add_item_to_playlist(widget: &str, input: Value) -> Result<String, VestaboardError> {
    let path = get_playlist_path();
    let mut playlist = Playlist::load_silent(&path)?;

    let id = playlist.add_widget(widget, input);
    playlist.save_silent(&path)?;

    log::info!("Added item {} ({}) to playlist", id, widget);
    Ok(id)
}

/// List all items in the playlist
pub fn list_playlist() -> Result<(), VestaboardError> {
    let path = get_playlist_path();
    let playlist = Playlist::load_silent(&path)?;

    if playlist.is_empty() {
        println!("Playlist is empty.");
        println!("Add items with: vbl playlist add <widget> [input]");
        return Ok(());
    }

    println!(
        "Playlist ({} items, {} second interval):",
        playlist.len(),
        playlist.interval_seconds
    );
    println!();

    for (index, item) in playlist.items.iter().enumerate() {
        let input_display = if item.input.is_null() {
            String::new()
        } else if let Some(s) = item.input.as_str() {
            format!(" \"{}\"", s)
        } else {
            format!(" {}", item.input)
        };

        println!(
            "  {}. [{}] {}{}",
            index + 1,
            item.id,
            item.widget,
            input_display
        );
    }

    println!();
    Ok(())
}

/// Remove an item from the playlist by ID
pub fn remove_item_from_playlist(id: &str) -> Result<(), VestaboardError> {
    let path = get_playlist_path();
    let mut playlist = Playlist::load_silent(&path)?;

    if !playlist.remove_item(id) {
        return Err(VestaboardError::validation_error(&format!(
            "Item '{}' not found in playlist",
            id
        )));
    }

    playlist.save_silent(&path)?;
    log::info!("Removed item {} from playlist", id);
    Ok(())
}

/// Clear all items from the playlist
pub fn clear_playlist() -> Result<(), VestaboardError> {
    let path = get_playlist_path();
    let mut playlist = Playlist::load_silent(&path)?;

    let count = playlist.len();
    playlist.clear();
    playlist.save_silent(&path)?;

    log::info!("Cleared {} items from playlist", count);
    Ok(())
}

/// Show the current playlist rotation interval
pub fn show_playlist_interval() -> Result<(), VestaboardError> {
    let path = get_playlist_path();
    let playlist = Playlist::load_silent(&path)?;

    println!("Current interval: {} seconds", playlist.interval_seconds);
    Ok(())
}

/// Set the playlist rotation interval
pub fn set_playlist_interval(seconds: u64) -> Result<(), VestaboardError> {
    if seconds < 60 {
        return Err(VestaboardError::validation_error(
            "Interval must be at least 60 seconds",
        ));
    }

    let path = get_playlist_path();
    let mut playlist = Playlist::load_silent(&path)?;

    playlist.interval_seconds = seconds;
    playlist.save_silent(&path)?;

    log::info!("Set playlist interval to {} seconds", seconds);
    Ok(())
}

/// Preview all items in the playlist (dry-run mode)
pub async fn preview_playlist() {
    let path = get_playlist_path();
    let playlist = match Playlist::load_silent(&path) {
        Ok(p) => p,
        Err(e) => {
            print_error(&e.to_user_message());
            return;
        }
    };

    if playlist.is_empty() {
        println!("Playlist is empty. Nothing to preview.");
        return;
    }

    println!(
        "Previewing {} playlist items ({} second interval):",
        playlist.len(),
        playlist.interval_seconds
    );
    println!();

    for (index, item) in playlist.items.iter().enumerate() {
        let input_display = if item.input.is_null() {
            String::new()
        } else if let Some(s) = item.input.as_str() {
            format!(" \"{}\"", s)
        } else {
            format!(" {}", item.input)
        };

        println!("--- Item {} of {}: {}{} ---", index + 1, playlist.len(), item.widget, input_display);

        // Execute widget and show preview
        let message = match execute_widget(&item.widget, &item.input).await {
            Ok(msg) => msg,
            Err(e) => {
                println!("  Error: {}", e.to_user_message());
                error_to_display_message(&e)
            }
        };

        // Display to console (dry-run)
        if let Err(e) = handle_message(message, MessageDestination::Console).await {
            println!("  Display error: {}", e.to_user_message());
        }

        println!();
    }

    println!("Preview complete.");
}

/// Run the playlist with interactive controls.
///
/// # Arguments
/// * `once` - If true, run through playlist once and exit
/// * `resume` - If true, resume from last saved position
/// * `start_index` - Optional starting index (0-based)
/// * `start_id` - Optional starting item ID
/// * `dry_run` - If true, display to console instead of Vestaboard
pub async fn run_playlist(
    once: bool,
    resume: bool,
    start_index: Option<usize>,
    start_id: Option<String>,
    dry_run: bool,
) -> Result<(), VestaboardError> {
    let playlist_path = get_playlist_path();
    let config = Config::load_silent().unwrap_or_default();
    let state_path = config.get_runtime_state_path();

    // Load playlist
    let playlist = Playlist::load_silent(&playlist_path)?;

    if playlist.is_empty() {
        println!("Playlist is empty. Add items with: vbl playlist add <widget>");
        return Ok(());
    }

    // Determine starting index
    let start_idx = match (start_index, start_id) {
        (Some(idx), _) => {
            if idx >= playlist.len() {
                return Err(VestaboardError::validation_error(&format!(
                    "Index {} is out of range (playlist has {} items)",
                    idx,
                    playlist.len()
                )));
            }
            idx
        }
        (_, Some(id)) => {
            playlist.find_index_by_id(&id).ok_or_else(|| {
                VestaboardError::validation_error(&format!("Item '{}' not found in playlist", id))
            })?
        }
        (None, None) if resume => {
            // Restore from saved state (--resume flag)
            let saved_state = crate::runtime_state::RuntimeState::load(&state_path);
            if saved_state.playlist_index < playlist.len() {
                log::info!("Resuming from saved index {}", saved_state.playlist_index);
                saved_state.playlist_index
            } else {
                log::info!("Saved index {} out of range, starting from 0", saved_state.playlist_index);
                0
            }
        }
        (None, None) => 0, // Default: start from beginning
    };

    // Acquire exclusive lock
    let _lock = InstanceLock::acquire("playlist")?;

    // Create runner
    let mut runner = PlaylistRunner::new(playlist, state_path, start_idx, once, dry_run);

    // Setup keyboard listener
    let mut keyboard = KeyboardListener::new()?;

    // Setup Ctrl+C handler
    let process_controller = ProcessController::new();
    process_controller.setup_signal_handler()?;

    // Show initial help
    println!("Press ? for help, q to quit.");

    // Start the runner
    runner.start();

    // Main loop
    loop {
        // Priority 1: Check for shutdown signal (Ctrl+C)
        if process_controller.should_shutdown() {
            log::info!("Shutdown requested, stopping playlist");
            println!("\nShutting down...");
            break;
        }

        // Priority 2: Check for keyboard input (non-blocking)
        if let Some(key) = keyboard.try_recv() {
            match runner.handle_key(key) {
                ControlFlow::Continue => {}
                ControlFlow::Exit => {
                    log::info!("User requested exit via keyboard");
                    break;
                }
            }
        }

        // Priority 3: Run one iteration of the runner
        match runner.run_iteration().await {
            Ok(ControlFlow::Continue) => {}
            Ok(ControlFlow::Exit) => {
                log::info!("Runner requested exit");
                break;
            }
            Err(e) => {
                log::error!("Runner error: {}", e);
                print_error(&e.to_user_message());
                // Continue running unless fatal - matches cycle.rs behavior
            }
        }

        // Small sleep to prevent busy-looping
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Cleanup
    runner.cleanup();
    print_success("Playlist stopped.");

    // Lock is automatically released when _lock is dropped (RAII)
    Ok(())
}
