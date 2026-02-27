// Tests for the UserSettings / config module.
use cookbook_gtk::config::{Theme, UserSettings};

#[test]
fn default_settings_are_system_theme() {
    let s = UserSettings::default();
    assert!(matches!(s.theme, Theme::System));
}

#[test]
fn settings_round_trip_toml() {
    let mut s = UserSettings::default();
    s.theme = Theme::Dark;
    s.data_dir = Some("/tmp/test_data".to_string());

    let encoded = toml::to_string(&s).expect("serialize to toml");
    let decoded: UserSettings = toml::from_str(&encoded).expect("deserialize from toml");

    assert!(matches!(decoded.theme, Theme::Dark));
    assert_eq!(decoded.data_dir.as_deref(), Some("/tmp/test_data"));
}

#[test]
fn effective_data_dir_falls_back_to_something() {
    let path = UserSettings::effective_data_dir();
    assert!(!path.as_os_str().is_empty());
}
