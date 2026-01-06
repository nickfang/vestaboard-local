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
