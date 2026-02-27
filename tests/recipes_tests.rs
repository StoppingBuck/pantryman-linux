// Pure engine tests (no GTK required).
use janus_engine::DataManager;
use std::path::PathBuf;

fn fixture_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("example/data")
}

#[test]
fn data_manager_loads_recipes() {
    let dm = DataManager::new(fixture_data_dir(), "test-device").unwrap();
    assert!(
        dm.get_all_recipes().len() >= 1,
        "expected at least one recipe in example/data"
    );
}

#[test]
fn search_recipes_returns_match() {
    let dm = DataManager::new(fixture_data_dir(), "test-device").unwrap();
    let results = dm.search_recipes("Lasagna");
    assert!(!results.is_empty(), "expected Lasagna to be found");
    assert_eq!(results[0].title, "Lasagna");
}
