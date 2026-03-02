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
    /// Stable per-device identifier used for per-device pantry files.
    /// Generated once on first run and persisted.
    #[serde(default)]
    pub device_id: Option<String>,
}

fn default_language() -> String {
    "system".to_string()
}

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            data_dir: None,
            language: "system".to_string(),
            theme: Theme::default(),
            device_id: None,
        }
    }
}

impl UserSettings {
    /// Returns this device's stable ID, generating and persisting one if needed.
    pub fn effective_device_id() -> String {
        let mut settings = Self::load();
        if let Some(id) = &settings.device_id {
            return id.clone();
        }
        let id = Self::generate_device_id();
        settings.device_id = Some(id.clone());
        settings.save();
        id
    }

    /// Generates a random 8-character alphanumeric ID prefixed with "kde-".
    /// Uses `/dev/urandom` on Linux with a timestamp fallback.
    fn generate_device_id() -> String {
        use std::io::Read;
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut bytes = [0u8; 8];
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            let _ = f.read_exact(&mut bytes);
        } else {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();
            for (i, b) in bytes.iter_mut().enumerate() {
                *b = ((nanos >> (i * 4)) & 0xFF) as u8;
            }
        }
        let mut id = String::from("kde-");
        for b in &bytes {
            id.push(CHARS[(*b as usize) % CHARS.len()] as char);
        }
        id
    }
}

impl UserSettings {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pantryman/user_settings.toml")
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
        assert_eq!(s.language, "system");
        assert_eq!(s.theme, Theme::System);
        assert!(s.data_dir.is_none());
    }

    #[test]
    fn test_settings_roundtrip() {
        let original = UserSettings {
            data_dir: Some("/tmp/test".to_string()),
            language: "de".to_string(),
            theme: Theme::Dark,
            device_id: Some("kde-testabcd".to_string()),
        };
        let serialized = toml::to_string(&original).expect("serialize");
        let loaded: UserSettings = toml::from_str(&serialized).expect("deserialize");
        assert_eq!(loaded.data_dir, original.data_dir);
        assert_eq!(loaded.language, original.language);
        assert_eq!(loaded.theme, original.theme);
    }
}
