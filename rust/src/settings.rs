use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::Write, path::PathBuf};
use thiserror::Error;
pub const APP_ID: &str = "com.goshapps.YubicoAuthenticator";
pub const APP_NAME: &str = "Gosh Yubico Authenticator";
pub const APP_WEBSITE: &str =
    "https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}
impl ThemeMode {
    pub fn css(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct IconPreference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_icon_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favicon_domain: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub maximized: bool,
}
impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 960.,
            height: 720.,
            maximized: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub theme_mode: ThemeMode,
    pub clipboard_timeout_seconds: u32,
    pub require_pin_on_launch: bool,
    pub icon_prefs: HashMap<String, IconPreference>,
    pub favicon_cache: HashMap<String, serde_json::Value>,
    pub allow_favicons: bool,
    pub window: WindowState,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            clipboard_timeout_seconds: 30,
            require_pin_on_launch: false,
            icon_prefs: HashMap::new(),
            favicon_cache: HashMap::new(),
            allow_favicons: false,
            window: WindowState::default(),
            extra: HashMap::new(),
        }
    }
}
/// Individual edits are merged into the worker's latest settings, so pending
/// saves from another control or window resize cannot replace unrelated fields.
pub enum SettingsUpdate {
    ThemeMode(ThemeMode),
    ClipboardTimeout(u32),
    RequirePinOnLaunch(bool),
    AllowFavicons(bool),
    Window(WindowState),
    Icon {
        key: String,
        preference: Option<IconPreference>,
    },
}
impl SettingsUpdate {
    pub(crate) fn apply_to(self, settings: &mut Settings) {
        match self {
            Self::ThemeMode(value) => settings.theme_mode = value,
            Self::ClipboardTimeout(value) => settings.clipboard_timeout_seconds = value,
            Self::RequirePinOnLaunch(value) => settings.require_pin_on_launch = value,
            Self::AllowFavicons(value) => settings.allow_favicons = value,
            Self::Window(value) => settings.window = value,
            Self::Icon { key, preference } => match preference {
                Some(value) => {
                    settings.icon_prefs.insert(key, value);
                }
                None => {
                    settings.icon_prefs.remove(&key);
                }
            },
        }
    }
}
impl Settings {
    pub fn icon_key(id: &[u8]) -> String {
        data_encoding::BASE64.encode(id)
    }
    pub fn validate(&self) -> Result<(), SettingsError> {
        if ![10, 20, 30, 60, 120].contains(&self.clipboard_timeout_seconds) {
            return Err(SettingsError::Invalid(
                "Clipboard timeout must be 10, 20, 30, 60, or 120 seconds".into(),
            ));
        }
        if !self.window.width.is_finite()
            || !self.window.height.is_finite()
            || !(360.0..=16384.).contains(&self.window.width)
            || !(400.0..=16384.).contains(&self.window.height)
        {
            return Err(SettingsError::Invalid("Invalid window dimensions".into()));
        }
        for icon in self.icon_prefs.values() {
            if let Some(domain) = &icon.favicon_domain {
                validated_domain(domain)?;
            }
        }
        Ok(())
    }
}
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Settings I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("Settings JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}
#[derive(Clone)]
pub struct SettingsStore {
    pub path: PathBuf,
}
impl SettingsStore {
    pub fn standard() -> Result<Self, SettingsError> {
        let dir = dirs::config_dir().ok_or_else(|| {
            SettingsError::Invalid("Cannot locate the OS configuration directory".into())
        })?;
        Ok(Self {
            path: dir.join("gosh-authenticator").join("settings.json"),
        })
    }
    pub fn load(&self) -> Result<Settings, SettingsError> {
        use std::io::Read;
        match fs::File::open(&self.path).and_then(|file| {
            let mut data = Vec::new();
            file.take(1024 * 1024 + 1).read_to_end(&mut data)?;
            Ok(data)
        }) {
            Ok(data) => {
                if data.len() > 1024 * 1024 {
                    return Err(SettingsError::Invalid(
                        "Settings exceed the 1 MiB limit".into(),
                    ));
                }
                let settings: Settings = serde_json::from_slice(&data)?;
                settings.validate()?;
                Ok(settings)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(e) => Err(e.into()),
        }
    }
    pub fn save(&self, value: &Settings) -> Result<(), SettingsError> {
        value.validate()?;
        // A corrupt/unreadable existing file must never be overwritten by defaults.
        self.load()?;
        let parent = self
            .path
            .parent()
            .ok_or_else(|| SettingsError::Invalid("Settings path has no parent".into()))?;
        fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(&serde_json::to_vec_pretty(value)?)?;
        file.write_all(b"\n")?;
        file.as_file().sync_all()?;
        file.persist(&self.path)
            .map_err(|e| SettingsError::Io(e.error))?;
        Ok(())
    }
}
pub fn cache_dir() -> Result<PathBuf, SettingsError> {
    dirs::cache_dir()
        .map(|d| d.join("gosh-authenticator"))
        .ok_or_else(|| SettingsError::Invalid("Cannot locate OS cache directory".into()))
}
/// ASCII hash filenames avoid Windows reserved names, casing and length limits.
pub fn favicon_filename(domain: &str) -> String {
    use sha2::Digest;
    format!(
        "{}.png",
        data_encoding::HEXLOWER.encode(&sha2::Sha256::digest(domain.as_bytes()))
    )
}
pub fn validated_domain(input: &str) -> Result<String, SettingsError> {
    let trimmed = input.trim();
    let parsed = url::Url::parse(if trimmed.starts_with("https://") {
        trimmed
    } else {
        return domain_only(trimmed);
    })
    .map_err(|_| SettingsError::Invalid("Enter a valid website hostname".into()))?;
    if parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some()
    {
        return Err(SettingsError::Invalid(
            "Use a website hostname without a path, port or credentials".into(),
        ));
    }
    domain_only(parsed.host_str().unwrap_or_default())
}
fn domain_only(input: &str) -> Result<String, SettingsError> {
    let domain = input.to_ascii_lowercase();
    if domain.len() > 253
        || !domain.contains('.')
        || domain.parse::<std::net::IpAddr>().is_ok()
        || domain.split('.').any(|p| {
            p.is_empty()
                || p.len() > 63
                || p.starts_with('-')
                || p.ends_with('-')
                || !p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return Err(SettingsError::Invalid(
            "Use a DNS hostname such as example.com".into(),
        ));
    }
    Ok(domain)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_json_roundtrip_preserves_unknown_fields() {
        let s:Settings=serde_json::from_str(r#"{"theme_mode":"dark","clipboard_timeout_seconds":60,"require_pin_on_launch":true,"icon_prefs":{"YWJj":{"custom_icon_key":"github"}},"future_setting":{"v":1}}"#).unwrap();
        s.validate().unwrap();
        assert_eq!(s.theme_mode, ThemeMode::Dark);
        assert!(!s.allow_favicons);
        let encoded = serde_json::to_value(&s).unwrap();
        assert_eq!(encoded["future_setting"]["v"], 1);
        assert_eq!(encoded["icon_prefs"]["YWJj"]["custom_icon_key"], "github");
    }
    #[test]
    fn unicode_path_atomic_save_and_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore {
            path: dir.path().join("配置 with spaces").join("settings.json"),
        };
        store.save(&Settings::default()).unwrap();
        let mut s = store.load().unwrap();
        s.theme_mode = ThemeMode::Dark;
        store.save(&s).unwrap();
        assert_eq!(store.load().unwrap(), s);
        fs::write(&store.path, b"{\r\n\"theme_mode\":\"light\"\r\n}").unwrap();
        assert_eq!(store.load().unwrap().theme_mode, ThemeMode::Light);
    }
    #[test]
    fn corrupt_file_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore {
            path: dir.path().join("settings.json"),
        };
        fs::write(&store.path, b"not JSON").unwrap();
        assert!(store.save(&Settings::default()).is_err());
        assert_eq!(fs::read(&store.path).unwrap(), b"not JSON");
    }
    #[test]
    fn domain_cannot_escape_cache() {
        for s in [
            "../file.png",
            "example.com/../../file",
            "https://example.com/private",
            "127.0.0.1",
            "user@example.com",
            "-bad.com",
        ] {
            assert!(validated_domain(s).is_err(), "{s}");
        }
        assert_eq!(
            validated_domain("https://Example.COM/").unwrap(),
            "example.com"
        );
    }
}
