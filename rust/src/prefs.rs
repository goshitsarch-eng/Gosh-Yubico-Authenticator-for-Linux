use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const APP_ID: &str = "com.github.gosh.gosh_yubikey_manager";
pub const APP_NAME: &str = "Gosh Yubikey Manager";
pub const APP_VERSION: &str = "1.2.0";
pub const APP_DEVELOPER: &str = "Goshitsarch";
pub const APP_WEBSITE: &str =
    "https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux";
pub const APP_ISSUES: &str =
    "https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux/issues";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn as_label(self) -> &'static str {
        match self {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }

    pub fn from_index(index: u32) -> Self {
        match index {
            1 => ThemeMode::Light,
            2 => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }

    pub fn index(self) -> u32 {
        match self {
            ThemeMode::System => 0,
            ThemeMode::Light => 1,
            ThemeMode::Dark => 2,
        }
    }
}

pub const CLIPBOARD_TIMEOUTS: [u32; 5] = [10, 20, 30, 60, 120];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IconPreference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_icon_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favicon_domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    #[serde(default)]
    pub theme_mode: ThemeMode,
    #[serde(default = "default_clipboard_timeout")]
    pub clipboard_timeout_seconds: u32,
    #[serde(default)]
    pub require_pin_on_launch: bool,
    #[serde(default)]
    pub icon_prefs: HashMap<String, IconPreference>,
    #[serde(default)]
    pub favicon_cache: HashMap<String, FaviconCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaviconCacheEntry {
    pub path: String,
    pub fetched_unix: i64,
}

fn default_clipboard_timeout() -> u32 {
    30
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            clipboard_timeout_seconds: 30,
            require_pin_on_launch: false,
            icon_prefs: HashMap::new(),
            favicon_cache: HashMap::new(),
        }
    }
}

impl Prefs {
    pub fn load() -> Self {
        let path = settings_path();
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, text);
        }
    }

    pub fn clipboard_timeout_index(&self) -> u32 {
        CLIPBOARD_TIMEOUTS
            .iter()
            .position(|v| *v == self.clipboard_timeout_seconds)
            .unwrap_or(2) as u32
    }

    pub fn set_clipboard_timeout_index(&mut self, index: u32) {
        if let Some(value) = CLIPBOARD_TIMEOUTS.get(index as usize) {
            self.clipboard_timeout_seconds = *value;
        }
    }

    pub fn icon_key(id: &[u8]) -> String {
        data_encoding::BASE64.encode(id)
    }
}

pub fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gosh-authenticator")
        .join("settings.json")
}

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gosh-authenticator")
}

pub fn apply_theme(mode: ThemeMode) {
    let manager = adw::StyleManager::default();
    let scheme = match mode {
        ThemeMode::System => adw::ColorScheme::Default,
        ThemeMode::Light => adw::ColorScheme::ForceLight,
        ThemeMode::Dark => adw::ColorScheme::ForceDark,
    };
    manager.set_color_scheme(scheme);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_roundtrip() {
        assert_eq!(ThemeMode::from_index(0), ThemeMode::System);
        assert_eq!(ThemeMode::from_index(1), ThemeMode::Light);
        assert_eq!(ThemeMode::from_index(2), ThemeMode::Dark);
        assert_eq!(ThemeMode::Dark.index(), 2);
    }

    #[test]
    fn clipboard_index() {
        let mut prefs = Prefs::default();
        assert_eq!(prefs.clipboard_timeout_seconds, 30);
        assert_eq!(prefs.clipboard_timeout_index(), 2);
        prefs.set_clipboard_timeout_index(4);
        assert_eq!(prefs.clipboard_timeout_seconds, 120);
    }
}
