use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(default)]
    pub data_dir: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub theme: Theme,
}

fn default_language() -> String {
    "en".to_string()
}

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            data_dir: None,
            language: "en".to_string(),
            theme: Theme::default(),
        }
    }
}

impl UserSettings {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cookbook-gtk/user_settings.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            toml::from_str(&content).unwrap_or_default()
        } else {
            UserSettings::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string(self) {
            let _ = std::fs::write(&path, content);
        }
    }

    /// Returns the effective data directory (from settings or default dev path).
    pub fn effective_data_dir() -> PathBuf {
        // Check environment variable first
        if let Ok(env_path) = std::env::var("COOKBOOK_DATA_DIR") {
            return PathBuf::from(env_path);
        }
        let settings = Self::load();
        if let Some(dir) = settings.data_dir {
            PathBuf::from(dir)
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("example")
                .join("data")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = UserSettings::default();
        assert_eq!(s.language, "en");
        assert_eq!(s.theme, Theme::System);
        assert!(s.data_dir.is_none());
    }

    #[test]
    fn test_settings_roundtrip() {
        let original = UserSettings {
            data_dir: Some("/tmp/test".to_string()),
            language: "de".to_string(),
            theme: Theme::Dark,
        };
        let serialized = toml::to_string(&original).expect("serialize");
        let loaded: UserSettings = toml::from_str(&serialized).expect("deserialize");
        assert_eq!(loaded.data_dir, original.data_dir);
        assert_eq!(loaded.language, original.language);
        assert_eq!(loaded.theme, original.theme);
    }
}
