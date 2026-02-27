use cookbook_gtk::ui_constants::*;

#[test]
fn default_margin_is_positive() {
    assert!(DEFAULT_MARGIN > 0);
}

#[test]
fn section_spacing_is_positive() {
    assert!(SECTION_SPACING > 0);
}

#[test]
fn window_dimensions_are_sensible() {
    assert!(DEFAULT_WINDOW_WIDTH >= 800);
    assert!(DEFAULT_WINDOW_HEIGHT >= 600);
    assert!(LIST_PANE_WIDTH > 0);
    assert!(SIDEBAR_WIDTH > 0);
}
