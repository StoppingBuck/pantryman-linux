// Integration tests for the App component.
// Most tests require a GTK display — run with `./dev.sh gtk-test-headless`.
#[cfg(test)]
mod tests {
    #[test]
    fn app_module_compiles() {
        // Smoke test: the module is accessible and core types are importable.
        let _: fn() = || {
            let _ = std::mem::size_of::<cookbook_gtk::app::Tab>();
        };
    }
}
