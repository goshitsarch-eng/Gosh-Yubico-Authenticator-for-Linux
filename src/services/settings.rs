use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application settings persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Theme mode: "light", "dark", or "system"
    pub theme_mode: String,
    /// Seconds before clearing clipboard (0 = never)
    pub clipboard_timeout: u64,
    /// Whether to require PIN on app launch
    pub pin_on_launch: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: "system".to_string(),
            clipboard_timeout: 30,
            pin_on_launch: false,
        }
    }
}

impl AppSettings {
    /// Get the settings file path
    fn settings_path() -> PathBuf {
        let dirs = directories::ProjectDirs::from("com", "gosh", "gosh-authenticator")
            .expect("Failed to determine config directory");
        let config_dir = dirs.config_dir();
        std::fs::create_dir_all(config_dir).ok();
        config_dir.join("settings.json")
    }

    /// Load settings from disk
    pub fn load() -> Self {
        let path = Self::settings_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save settings to disk
    pub fn save(&self) {
        let path = Self::settings_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            if let Err(e) = std::fs::write(&path, content) {
                log::error!("Failed to save settings: {}", e);
            }
        }
    }
}
