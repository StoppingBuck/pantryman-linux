use relm4::gtk;
use gtk::prelude::*;
use std::fs;
use std::path::Path;

/// Small pie chart showing `ratio` (0.0–1.0) of required ingredients in stock.
pub fn build_coverage_pie(ratio: f64, tooltip: &str) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.set_size_request(18, 18);
    area.set_valign(gtk::Align::Center);
    area.set_tooltip_text(Some(tooltip));
    area.set_draw_func(move |_, cr, w, h| {
        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let r  = cx.min(cy) - 0.5;
        let tau = 2.0 * std::f64::consts::PI;

        // Red background (missing portion)
        cr.arc(cx, cy, r, 0.0, tau);
        cr.set_source_rgb(0.753, 0.11, 0.157); // Adwaita #C01C28
        let _ = cr.fill();

        // Green filled portion (in-stock), clockwise from top
        if ratio > 0.0 {
            let end = -tau / 4.0 + ratio * tau;
            cr.move_to(cx, cy);
            cr.arc(cx, cy, r, -tau / 4.0, end);
            cr.close_path();
            cr.set_source_rgb(0.149, 0.635, 0.412); // Adwaita #26A269
            let _ = cr.fill();
        }

        // Thin border
        cr.arc(cx, cy, r, 0.0, tau);
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.15);
        cr.set_line_width(0.8);
        let _ = cr.stroke();
    });
    area
}

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
