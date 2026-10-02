use crate::{
    core::credential::{Credential, CredentialId, NewCredential},
    settings::{Settings, SettingsUpdate},
};
use zeroize::Zeroizing;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Connecting,
    Disconnected,
    Locked,
    Ready,
}
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub version: (u8, u8, u8),
    pub id: [u8; 8],
    pub has_password: bool,
}
pub enum Command {
    Connect,
    Refresh,
    Authenticate(Zeroizing<String>),
    Calculate(CredentialId),
    Add(NewCredential),
    Delete(CredentialId),
    SetPassword(Zeroizing<String>),
    Disconnect,
    Import(std::path::PathBuf),
    Copy(CredentialId),
    UpdateSettings(SettingsUpdate),
    FetchIcon(String),
    Shutdown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Added,
    Deleted,
    Unlocked,
    PasswordChanged,
    PasswordRemoved,
    Copied,
    IconDownloaded,
    Saved,
}
impl Outcome {
    pub fn message(self) -> &'static str {
        match self {
            Self::Added => "Credential added",
            Self::Deleted => "Credential deleted",
            Self::Unlocked => "YubiKey unlocked",
            Self::PasswordChanged => "OATH password updated",
            Self::PasswordRemoved => "Password protection removed",
            Self::Copied => "Code copied; clipboard will clear after the selected timeout",
            Self::IconDownloaded => "Favicon downloaded; save the icon preference to apply it",
            Self::Saved => "Saved",
        }
    }
}
pub enum Event {
    Connected(DeviceInfo, bool),
    Disconnected,
    Credentials(Vec<Credential>),
    Imported(NewCredential),
    SettingsSaved(Settings),
    IconReady(String, Vec<u8>),
    Success(Outcome),
    Error(String),
    Busy(bool),
    Touch(CredentialId),
    Cancelled,
}
#[derive(Clone)]
pub struct State {
    pub connection: Connection,
    pub device: Option<DeviceInfo>,
    pub credentials: Vec<Credential>,
    pub settings: Settings,
    pub busy: bool,
    pub message: Option<String>,
    pub error: bool,
    pub search: String,
    pub icons: std::collections::HashMap<String, String>,
}
impl State {
    pub fn new(settings: Settings) -> Self {
        Self {
            connection: Connection::Connecting,
            device: None,
            credentials: Vec::new(),
            settings,
            busy: false,
            message: None,
            error: false,
            search: String::new(),
            icons: std::collections::HashMap::new(),
        }
    }
    pub fn apply(&mut self, event: Event) {
        match event {
            Event::Connected(info, locked) => {
                self.connection = if locked {
                    Connection::Locked
                } else {
                    Connection::Ready
                };
                self.device = Some(info);
                self.credentials.clear();
            }
            Event::Disconnected => {
                self.connection = Connection::Disconnected;
                self.device = None;
                self.credentials.clear();
            }
            Event::Credentials(c) => {
                self.credentials = c;
                self.connection = Connection::Ready;
            }
            Event::SettingsSaved(s) => {
                self.settings = s;
            }
            Event::Success(msg) => {
                self.message = Some(msg.message().into());
                self.error = false;
            }
            Event::Error(msg) => {
                self.message = Some(msg);
                self.error = true;
            }
            Event::Busy(b) => self.busy = b,
            Event::IconReady(domain, png) => {
                self.icons.insert(
                    domain,
                    format!(
                        "data:image/png;base64,{}",
                        data_encoding::BASE64.encode(&png)
                    ),
                );
            }
            Event::Imported(_) | Event::Touch(_) => {}
            Event::Cancelled => {
                self.message = Some("Operation cancelled".into());
                self.error = false;
            }
        }
    }
    pub fn filtered(&self) -> Vec<Credential> {
        let q = self.search.trim().to_lowercase();
        self.credentials
            .iter()
            .filter(|c| c.display_name().to_lowercase().contains(&q))
            .cloned()
            .collect()
    }
}
pub fn unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
pub fn should_clear_clipboard(owned: &str, current: &str) -> bool {
    owned == current
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disconnect_removes_codes_and_identity() {
        let mut s = State::new(Settings::default());
        s.apply(Event::Connected(
            DeviceInfo {
                version: (5, 7, 1),
                id: [8; 8],
                has_password: true,
            },
            true,
        ));
        assert_eq!(s.connection, Connection::Locked);
        s.apply(Event::Disconnected);
        assert!(s.credentials.is_empty());
        assert!(s.device.is_none());
    }
    #[test]
    fn errors_are_visible_and_recoverable() {
        let mut s = State::new(Settings::default());
        s.apply(Event::Error("Permission denied".into()));
        assert!(s.error);
        assert!(s.message.as_ref().unwrap().contains("Permission"));
        s.apply(Event::Success(Outcome::Saved));
        assert!(!s.error);
    }
    #[test]
    fn clipboard_never_clears_replacement() {
        assert!(should_clear_clipboard("123456", "123456"));
        assert!(!should_clear_clipboard("123456", "newer user text"));
    }
}
