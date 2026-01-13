//! Tests for the playlist module.

use crate::playlist::{Playlist, PlaylistItem};
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

// --- PlaylistItem tests ---

#[test]
fn test_playlist_item_creation() {
  let item = PlaylistItem {
    id: "abc1".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  };
  assert_eq!(item.id, "abc1");
  assert_eq!(item.widget, "weather");
}

#[test]
fn test_playlist_item_creation_generates_id() {
  let item = PlaylistItem::new("weather".to_string(), json!(null));
  assert!(!item.id.is_empty());
  assert_eq!(item.widget, "weather");
}

#[test]
fn test_playlist_item_serializes_to_json() {
  let item = PlaylistItem {
    id: "abc1".to_string(),
    widget: "text".to_string(),
    input: json!("hello world"),
  };
  let serialized = serde_json::to_string(&item).unwrap();
  assert!(serialized.contains("\"widget\":\"text\""));
  assert!(serialized.contains("\"input\":\"hello world\""));
}

#[test]
fn test_playlist_item_deserializes_from_json() {
  let json_str = r#"{"id":"xyz9","widget":"weather","input":null}"#;
  let item: PlaylistItem = serde_json::from_str(json_str).unwrap();
  assert_eq!(item.id, "xyz9");
  assert_eq!(item.widget, "weather");
}

#[test]
fn test_playlist_item_deserializes_without_id_gets_generated() {
  let json_str = r#"{"widget":"weather","input":null}"#;
  let item: PlaylistItem = serde_json::from_str(json_str).unwrap();
  assert!(!item.id.is_empty());
  assert_eq!(item.widget, "weather");
}

// --- Playlist struct tests ---

#[test]
fn test_playlist_creation_with_defaults() {
  let playlist = Playlist::default();
  assert!(playlist.items.is_empty());
  assert_eq!(playlist.interval_seconds, 300);
}

#[test]
fn test_playlist_default_interval_is_300() {
  let playlist = Playlist::default();
  assert_eq!(playlist.interval_seconds, 300);
}

// --- CRUD operations tests ---

#[test]
fn test_playlist_add_item() {
  let mut playlist = Playlist::default();
  let item = PlaylistItem {
    id: "abc1".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  };
  playlist.add_item(item);
  assert_eq!(playlist.items.len(), 1);
  assert_eq!(playlist.items[0].widget, "weather");
}

#[test]
fn test_playlist_add_multiple_items_preserves_order() {
  let mut playlist = Playlist::default();
  playlist.add_item(PlaylistItem {
    id: "a".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  });
  playlist.add_item(PlaylistItem {
    id: "b".to_string(),
    widget: "text".to_string(),
    input: json!("hello"),
  });
  playlist.add_item(PlaylistItem {
    id: "c".to_string(),
    widget: "sat-word".to_string(),
    input: json!(null),
  });
  assert_eq!(playlist.items.len(), 3);
  assert_eq!(playlist.items[0].id, "a");
  assert_eq!(playlist.items[1].id, "b");
  assert_eq!(playlist.items[2].id, "c");
}

#[test]
fn test_playlist_add_widget() {
  let mut playlist = Playlist::default();
  let id = playlist.add_widget("weather", json!(null));
  assert!(!id.is_empty());
  assert_eq!(playlist.items.len(), 1);
  assert_eq!(playlist.items[0].widget, "weather");
  assert_eq!(playlist.items[0].id, id);
}

#[test]
fn test_playlist_remove_item_by_id() {
  let mut playlist = Playlist::default();
  playlist.add_item(PlaylistItem {
    id: "abc1".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  });
  playlist.add_item(PlaylistItem {
    id: "def2".to_string(),
    widget: "text".to_string(),
    input: json!("hello"),
  });

  let removed = playlist.remove_item("abc1");
  assert!(removed);
  assert_eq!(playlist.items.len(), 1);
  assert_eq!(playlist.items[0].id, "def2");
}

#[test]
fn test_playlist_remove_nonexistent_returns_false() {
  let mut playlist = Playlist::default();
  playlist.add_item(PlaylistItem {
    id: "abc1".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  });

  let removed = playlist.remove_item("nonexistent");
  assert!(!removed);
  assert_eq!(playlist.items.len(), 1);
}

#[test]
fn test_playlist_remove_from_empty_returns_false() {
  let mut playlist = Playlist::default();
  assert!(playlist.is_empty());

  let removed = playlist.remove_item("any_id");
  assert!(!removed);
  assert!(playlist.is_empty());
}

#[test]
fn test_playlist_is_empty() {
  let playlist = Playlist::default();
  assert!(playlist.is_empty());

  let mut playlist_with_items = Playlist::default();
  playlist_with_items.add_item(PlaylistItem {
    id: "abc1".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  });
  assert!(!playlist_with_items.is_empty());
}

#[test]
fn test_playlist_len() {
  let mut playlist = Playlist::default();
  assert_eq!(playlist.len(), 0);

  playlist.add_widget("weather", json!(null));
  assert_eq!(playlist.len(), 1);

  playlist.add_widget("text", json!("hello"));
  assert_eq!(playlist.len(), 2);
}

#[test]
fn test_playlist_clear() {
  let mut playlist = Playlist::default();
  playlist.add_widget("weather", json!(null));
  playlist.add_widget("text", json!("hello"));
  assert_eq!(playlist.len(), 2);

  playlist.clear();
  assert!(playlist.is_empty());
}

#[test]
fn test_playlist_get_item() {
  let mut playlist = Playlist::default();
  playlist.add_item(PlaylistItem {
    id: "abc1".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  });

  let item = playlist.get_item("abc1");
  assert!(item.is_some());
  assert_eq!(item.unwrap().widget, "weather");

  let not_found = playlist.get_item("nonexistent");
  assert!(not_found.is_none());
}

#[test]
fn test_playlist_get_item_by_index() {
  let mut playlist = Playlist::default();
  playlist.add_item(PlaylistItem {
    id: "a".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  });
  playlist.add_item(PlaylistItem {
    id: "b".to_string(),
    widget: "text".to_string(),
    input: json!("hello"),
  });

  let item = playlist.get_item_by_index(0);
  assert!(item.is_some());
  assert_eq!(item.unwrap().id, "a");

  let item = playlist.get_item_by_index(1);
  assert!(item.is_some());
  assert_eq!(item.unwrap().id, "b");

  let item = playlist.get_item_by_index(2);
  assert!(item.is_none());
}

#[test]
fn test_playlist_find_index_by_id() {
  let mut playlist = Playlist::default();
  playlist.add_item(PlaylistItem {
    id: "a".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  });
  playlist.add_item(PlaylistItem {
    id: "b".to_string(),
    widget: "text".to_string(),
    input: json!("hello"),
  });

  assert_eq!(playlist.find_index_by_id("a"), Some(0));
  assert_eq!(playlist.find_index_by_id("b"), Some(1));
  assert_eq!(playlist.find_index_by_id("c"), None);
}

// --- Interval validation tests ---

#[test]
fn test_playlist_validate_interval_rejects_under_60() {
  let mut playlist = Playlist::default();
  playlist.interval_seconds = 59;
  let result = playlist.validate_interval();
  assert!(result.is_err());
}

#[test]
fn test_playlist_validate_interval_accepts_60() {
  let mut playlist = Playlist::default();
  playlist.interval_seconds = 60;
  let result = playlist.validate_interval();
  assert!(result.is_ok());
}

#[test]
fn test_playlist_validate_interval_accepts_300() {
  let playlist = Playlist::default();
  let result = playlist.validate_interval();
  assert!(result.is_ok());
}

// --- File persistence tests ---

#[test]
fn test_playlist_save_and_load() {
  let mut playlist = Playlist::default();
  playlist.interval_seconds = 120;
  playlist.add_item(PlaylistItem {
    id: "abc1".to_string(),
    widget: "weather".to_string(),
    input: json!(null),
  });

  let temp_file = NamedTempFile::new().unwrap();
  let path = temp_file.path();

  playlist.save(path).unwrap();
  let loaded = Playlist::load(path).unwrap();

  assert_eq!(loaded.interval_seconds, 120);
  assert_eq!(loaded.items.len(), 1);
  assert_eq!(loaded.items[0].widget, "weather");
}

#[test]
fn test_playlist_load_nonexistent_returns_default() {
  let path = std::path::Path::new("/nonexistent/path/playlist.json");
  let playlist = Playlist::load(path).unwrap();
  assert!(playlist.items.is_empty());
  assert_eq!(playlist.interval_seconds, 300);
}

#[test]
fn test_playlist_load_invalid_json_returns_error() {
  let mut temp_file = NamedTempFile::new().unwrap();
  writeln!(temp_file, "not valid json {{{{").unwrap();

  let result = Playlist::load(temp_file.path());
  assert!(result.is_err());
}

#[test]
fn test_playlist_load_empty_file_returns_default() {
  let temp_file = NamedTempFile::new().unwrap();
  let playlist = Playlist::load(temp_file.path()).unwrap();
  assert!(playlist.items.is_empty());
}

#[test]
fn test_playlist_save_silent_and_load_silent() {
  let mut playlist = Playlist::default();
  playlist.interval_seconds = 180;
  playlist.add_widget("text", json!("test message"));

  let temp_file = NamedTempFile::new().unwrap();
  let path = temp_file.path();

  playlist.save_silent(path).unwrap();
  let loaded = Playlist::load_silent(path).unwrap();

  assert_eq!(loaded.interval_seconds, 180);
  assert_eq!(loaded.items.len(), 1);
}
