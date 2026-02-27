use relm4::gtk;
use gtk::prelude::*;
use std::fs;
use std::path::Path;

/// Remove all children from a Box widget.
pub fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

/// Remove all rows from a ListBox widget.
pub fn clear_list_box(container: &gtk::ListBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

/// Ensures the required data directory structure exists, creating subdirectories
/// and a minimal pantry.yaml if they are missing.
pub fn validate_and_create_data_dir<P: AsRef<Path>>(data_dir: P) {
    let data_dir = data_dir.as_ref();
    for subdir in &["ingredients", "recipes", "recipes/img"] {
        let path = data_dir.join(subdir);
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
    }
    let pantry = data_dir.join("pantry.yaml");
    if !pantry.exists() {
        let _ = fs::write(&pantry, "version: 1\nitems: []\n");
    }
}
