use std::path::Path;
use tempfile::tempdir;

#[test]
fn validate_and_create_data_dir_creates_structure() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("new_data");

    cookbook_gtk::utils::validate_and_create_data_dir(&dir);

    assert!(dir.join("ingredients").exists(), "ingredients/ missing");
    assert!(dir.join("recipes").exists(), "recipes/ missing");
    assert!(dir.join("pantry.yaml").exists(), "pantry.yaml missing");
}

#[test]
fn validate_and_create_data_dir_is_idempotent() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("data");

    cookbook_gtk::utils::validate_and_create_data_dir(&dir);
    // Second call must not panic or overwrite existing files.
    cookbook_gtk::utils::validate_and_create_data_dir(&dir);

    assert!(Path::new(&dir.join("pantry.yaml")).exists());
}
