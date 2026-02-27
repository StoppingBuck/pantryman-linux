// Full UI integration tests require a display server (Wayland/X11).
// Run headlessly with:  ./dev.sh gtk-test-headless
//
// All tests here are marked #[ignore] so `cargo test` passes in CI without a display.

#[test]
#[ignore = "requires display"]
fn app_starts_without_crashing() {
    // TODO: launch RelmApp, verify it initialises, quit immediately.
}
