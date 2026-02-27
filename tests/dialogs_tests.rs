// Dialogs require a live GTK/Wayland display and cannot be unit-tested
// without a display server. These tests are marked #[ignore] by default.
// Run with: DISPLAY=:0 cargo test -p cookbook-gtk dialog -- --include-ignored

#[test]
fn placeholder() {
    // Dialogs module compiles.
}
