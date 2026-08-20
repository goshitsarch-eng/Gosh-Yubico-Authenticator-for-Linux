use std::cell::{Cell, RefCell};
use std::time::Instant;

use crate::clipboard::ClipboardService;
use crate::prefs::Prefs;
use gosh_authenticator_core::core::credential::{Credential, CredentialId};
use gosh_authenticator_core::services::YubiKeyService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
}

pub struct AppState {
    pub service: YubiKeyService,
    pub prefs: RefCell<Prefs>,
    pub clipboard: ClipboardService,
    pub credentials: RefCell<Vec<Credential>>,
    pub connection: Cell<ConnectionStatus>,
    pub version: Cell<(u8, u8, u8)>,
    pub has_device: Cell<bool>,
    pub device_id: Cell<[u8; 8]>,
    pub search: RefCell<String>,
    pub copied_until: RefCell<Option<(CredentialId, Instant)>>,
    pub last_period_slot: Cell<u64>,
    pub pin_open: Cell<bool>,
    pub touch_open: Cell<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            service: YubiKeyService::new(),
            prefs: RefCell::new(Prefs::load()),
            clipboard: ClipboardService::new(),
            credentials: RefCell::new(Vec::new()),
            connection: Cell::new(ConnectionStatus::Connecting),
            version: Cell::new((0, 0, 0)),
            has_device: Cell::new(false),
            device_id: Cell::new([0; 8]),
            search: RefCell::new(String::new()),
            copied_until: RefCell::new(None),
            last_period_slot: Cell::new(0),
            pin_open: Cell::new(false),
            touch_open: Cell::new(false),
        }
    }

    pub fn filtered_credentials(&self) -> Vec<Credential> {
        let query = self.search.borrow().to_ascii_lowercase();
        self.credentials
            .borrow()
            .iter()
            .filter(|cred| {
                if query.is_empty() {
                    return true;
                }
                cred.display_name().to_ascii_lowercase().contains(&query)
                    || cred.account.to_ascii_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    pub fn totp_progress(period: u32) -> f64 {
        let period = if period == 0 { 30 } else { period as u64 };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        let elapsed = now % period as f64;
        (period as f64 - elapsed) / period as f64
    }

    pub fn period_slot(period: u32) -> u64 {
        let period = if period == 0 { 30 } else { period as u64 };
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() / period)
            .unwrap_or(0)
    }
}
