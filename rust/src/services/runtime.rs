use crate::{
    app::{unix_time, Command, DeviceInfo, Event, Outcome},
    core::{
        credential::{Credential, CredentialId},
        yubikey::{oath::Calculation, OathSession, YubiKeyError},
    },
    settings::{Settings, SettingsStore},
};
use async_channel::Receiver;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, SyncSender},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
#[derive(Clone)]
pub struct Runtime {
    sender: SyncSender<Queued>,
    pub events: Receiver<Event>,
    cancel: Arc<AtomicU64>,
    #[cfg(feature = "desktop")]
    clipboard: super::clipboard::ClipboardService,
}
impl Runtime {
    pub fn start(store: SettingsStore) -> Self {
        let (tx, rx) = mpsc::sync_channel::<Queued>(16);
        let (events_tx, events) = async_channel::bounded(64);
        let cancel = Arc::new(AtomicU64::new(0));
        let stopped = cancel.clone();
        #[cfg(feature = "desktop")]
        let clipboard = super::clipboard::ClipboardService::start(events_tx.clone());
        #[cfg(feature = "desktop")]
        let worker_clipboard = clipboard.clone();
        thread::spawn(move || {
            let settings = match store.load() {
                Ok(s) => s,
                Err(e) => {
                    let _ = events_tx.send_blocking(Event::Error(e.to_string()));
                    Settings::default()
                }
            };
            let mut worker = Worker {
                session: None,
                credentials: Vec::new(),
                settings,
                store,
                tx: events_tx,
                cancel: stopped,
                reconnect_enabled: true,
                generation: 0,
                #[cfg(feature = "desktop")]
                clipboard: worker_clipboard,
            };
            worker.load_icons();
            worker.connect(false);
            let mut slot = unix_time();
            let mut reconnect = Instant::now();
            loop {
                match rx.recv_timeout(Duration::from_millis(250)) {
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Ok(c) => {
                        if matches!(c.command, Command::Shutdown) {
                            break;
                        }
                        worker.handle(c);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }

                let now = unix_time();
                if worker.session.is_some() && reconnect.elapsed() > Duration::from_secs(2) {
                    worker.check_presence();
                    reconnect = Instant::now();
                }
                if worker.reconnect_enabled
                    && worker.session.is_none()
                    && reconnect.elapsed() > Duration::from_secs(2)
                {
                    worker.connect(true);
                    reconnect = Instant::now();
                }
                if now != slot {
                    slot = now;
                    if worker.session.as_ref().is_some_and(|s| !s.requires_auth())
                        && worker.credentials.iter().any(|c| {
                            c.is_totp()
                                && !c.touch_required
                                && c.valid_until.is_some_and(|t| t <= now)
                        })
                    {
                        worker.refresh();
                    }
                }
            }
        });
        Self {
            sender: tx,
            events,
            cancel,
            #[cfg(feature = "desktop")]
            clipboard,
        }
    }
    pub fn send(&self, command: Command) -> Result<(), String> {
        self.sender
            .try_send(Queued {
                command,
                generation: self.cancel.load(Ordering::Acquire),
            })
            .map_err(|e| match e {
                mpsc::TrySendError::Full(_) => "The device queue is busy; try again shortly".into(),
                mpsc::TrySendError::Disconnected(_) => "The device service stopped".into(),
            })
    }
    pub async fn shutdown(&self) {
        self.cancel();
        let _ = self.send(Command::Shutdown);
        #[cfg(feature = "desktop")]
        self.clipboard.stop().await;
    }
    pub fn cancel(&self) {
        self.cancel.fetch_add(1, Ordering::AcqRel);
    }
}
struct Queued {
    command: Command,
    generation: u64,
}
struct Worker {
    session: Option<OathSession>,
    credentials: Vec<Credential>,
    settings: Settings,
    store: SettingsStore,
    tx: async_channel::Sender<Event>,
    cancel: Arc<AtomicU64>,
    reconnect_enabled: bool,
    generation: u64,
    #[cfg(feature = "desktop")]
    clipboard: super::clipboard::ClipboardService,
}
impl Worker {
    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Acquire) != self.generation
    }
    fn check_presence(&mut self) {
        if self
            .session
            .as_ref()
            .is_some_and(|s| !matches!(s.is_present(), Ok(true)))
        {
            self.session = None;
            self.credentials.clear();
            self.emit(Event::Disconnected);
            #[cfg(feature = "desktop")]
            self.clipboard.clear();
        }
    }
    fn emit(&self, event: Event) {
        let _ = self.tx.send_blocking(event);
    }
    fn handle(&mut self, queued: Queued) {
        self.generation = queued.generation;
        let command = queued.command;
        if matches!(command, Command::Calculate(_) | Command::Import(_)) && self.cancelled() {
            self.emit(Event::Cancelled);
            return;
        }
        self.emit(Event::Busy(true));
        let result = self.execute(command);
        if let Err(e) = result {
            if matches!(e, YubiKeyError::NoDevice | YubiKeyError::Disconnected) {
                self.session = None;
                self.credentials.clear();
                self.emit(Event::Disconnected);
            }
            self.emit(Event::Error(e.to_string()));
        }
        self.emit(Event::Busy(false));
    }
    fn execute(&mut self, command: Command) -> Result<(), YubiKeyError> {
        match command {
            Command::Connect => {
                self.reconnect_enabled = true;
                self.connect(false);
            }
            Command::Refresh => {
                self.reconnect_enabled = true;
                if self.session.is_none() {
                    self.connect(false);
                } else {
                    self.refresh();
                }
            }
            Command::Authenticate(password) => {
                self.session
                    .as_mut()
                    .ok_or(YubiKeyError::NoDevice)?
                    .validate(&password)?;
                self.emit(Event::Success(Outcome::Unlocked));
                self.refresh();
            }
            Command::Add(new) => {
                self.session
                    .as_ref()
                    .ok_or(YubiKeyError::NoDevice)?
                    .put(&new)?;
                self.emit(Event::Success(Outcome::Added));
                self.refresh();
            }
            Command::Delete(id) => {
                let cred = self.credential(&id)?.clone();
                self.session
                    .as_ref()
                    .ok_or(YubiKeyError::NoDevice)?
                    .delete_credential(&cred)?;
                self.emit(Event::Success(Outcome::Deleted));
                self.refresh();
            }
            Command::Calculate(id) => {
                let cred = self.credential(&id)?.clone();
                if cred.touch_required {
                    self.emit(Event::Touch(id.clone()));
                }
                let (code, digits) = self
                    .session
                    .as_ref()
                    .ok_or(YubiKeyError::NoDevice)?
                    .calculate(&cred, None)?;
                if self.cancelled() {
                    self.emit(Event::Cancelled);
                } else {
                    let now = unix_time();
                    if let Some(c) = self.credentials.iter_mut().find(|c| c.id == id) {
                        c.set_code(code, digits);
                        c.valid_until = if c.is_totp() {
                            Some((now / u64::from(c.period) + 1) * u64::from(c.period))
                        } else {
                            None
                        };
                    }
                    self.emit(Event::Credentials(self.credentials.clone()));
                }
            }
            Command::SetPassword(password) => {
                let session = self.session.as_mut().ok_or(YubiKeyError::NoDevice)?;
                if !password.is_empty() && password.chars().count() < 4 {
                    return Err(YubiKeyError::Generic(
                        "Password must contain at least four characters".into(),
                    ));
                }
                session.set_code(&password)?;
                let info = DeviceInfo {
                    version: session.version(),
                    id: session.device_id(),
                    has_password: session.has_password(),
                };
                self.emit(Event::Connected(info, false));
                self.emit(Event::Success(if password.is_empty() {
                    Outcome::PasswordRemoved
                } else {
                    Outcome::PasswordChanged
                }));
                self.refresh();
            }
            Command::Disconnect => {
                self.reconnect_enabled = false;
                #[cfg(feature = "desktop")]
                self.clipboard.clear();
                self.session = None;
                self.credentials.clear();
                self.emit(Event::Disconnected);
            }
            Command::Import(path) => {
                let result = crate::qr::scan_qr_file(&path);
                if self.cancelled() {
                    self.emit(Event::Cancelled);
                } else {
                    match result {
                        Ok(new) => self.emit(Event::Imported(new)),
                        Err(e) => self.emit(Event::Error(e.to_string())),
                    }
                }
            }
            Command::UpdateSettings(settings) => {
                self.store
                    .save(&settings)
                    .map_err(|e| YubiKeyError::Generic(e.to_string()))?;
                self.settings = settings;
                self.emit(Event::SettingsSaved(self.settings.clone()));
                self.load_icons();
            }
            Command::Copy(id) => {
                #[cfg(feature = "desktop")]
                self.copy(&id)?;
                #[cfg(not(feature = "desktop"))]
                return Err(YubiKeyError::Generic(format!(
                    "Desktop clipboard feature is disabled for {}",
                    id.as_str()
                )));
            }
            Command::FetchIcon(domain) => {
                #[cfg(feature = "desktop")]
                self.fetch_icon(&domain)?;
                #[cfg(not(feature = "desktop"))]
                return Err(YubiKeyError::Generic(format!(
                    "Desktop favicon feature is disabled for {domain}"
                )));
            }
            Command::Shutdown => {}
        }
        Ok(())
    }
    fn credential(&self, id: &CredentialId) -> Result<&Credential, YubiKeyError> {
        self.credentials
            .iter()
            .find(|c| c.id == *id)
            .ok_or_else(|| {
                YubiKeyError::CredentialNotFound("Credential is no longer available".into())
            })
    }
    fn connect(&mut self, silent: bool) {
        self.session = None;
        self.credentials.clear();
        match OathSession::connect() {
            Ok(session) => {
                let locked = session.requires_auth();
                let info = DeviceInfo {
                    version: session.version(),
                    id: session.device_id(),
                    has_password: session.has_password(),
                };
                self.session = Some(session);
                self.emit(Event::Connected(info, locked));
                if !locked {
                    self.refresh();
                }
            }
            Err(e) => {
                if !silent {
                    self.emit(Event::Disconnected);
                    if !matches!(e, YubiKeyError::NoDevice) {
                        self.emit(Event::Error(format!("Cannot reach the smart-card service: {e}. On Linux, start pcscd; on Windows, enable Smart Card; macOS provides PC/SC.")));
                    }
                }
            }
        }
    }
    fn refresh(&mut self) {
        let result = (|| {
            let session = self.session.as_ref().ok_or(YubiKeyError::NoDevice)?;
            let mut list = session.list_credentials()?;
            let now = unix_time();
            for (id, result) in session.calculate_all(Some(now))? {
                if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                    match result {
                        Calculation::Code(code, digits) => {
                            let (code, digits) = if c.period != 30 {
                                session.calculate(c, Some(now))?
                            } else {
                                (code, digits)
                            };
                            c.set_code(code, digits);
                            c.valid_until =
                                Some((now / u64::from(c.period) + 1) * u64::from(c.period));
                        }
                        Calculation::Touch => {
                            c.touch_required = true;
                            if let Some(old) = self.credentials.iter().find(|old| {
                                old.id == id && old.valid_until.is_some_and(|t| t > now)
                            }) {
                                c.code = old.code.clone();
                                c.digits = old.digits;
                                c.valid_until = old.valid_until;
                            }
                        }
                        Calculation::Hotp => {
                            c.touch_required = false;
                            if let Some(old) = self.credentials.iter().find(|old| old.id == id) {
                                c.code = old.code.clone();
                                c.digits = old.digits;
                            }
                        }
                    }
                }
            }
            Ok::<_, YubiKeyError>(list)
        })();
        match result {
            Ok(list) => {
                self.credentials = list;
                self.emit(Event::Credentials(self.credentials.clone()));
            }
            Err(e) => {
                if matches!(e, YubiKeyError::Disconnected | YubiKeyError::NoDevice) {
                    self.session = None;
                    self.credentials.clear();
                    self.emit(Event::Disconnected);
                } else if matches!(e, YubiKeyError::AuthenticationRequired) {
                    self.connect(false);
                }
                self.emit(Event::Error(e.to_string()));
            }
        }
    }
    fn load_icons(&self) {
        for pref in self.settings.icon_prefs.values() {
            if let Some(domain) = &pref.favicon_domain {
                self.load_icon(domain);
            }
        }
    }
    fn load_icon(&self, domain: &str) {
        let Ok(domain) = crate::settings::validated_domain(domain) else {
            return;
        };
        let Ok(dir) = crate::settings::cache_dir() else {
            return;
        };
        let folder = dir.join("favicons");
        let hashed = folder.join(crate::settings::favicon_filename(&domain));
        let path = if hashed.exists() {
            hashed
        } else {
            folder.join(format!("{domain}.png"))
        };
        if std::fs::metadata(&path).is_ok_and(|m| m.len() <= 256 * 1024) {
            if let Ok(png) = std::fs::read(path) {
                if let Ok(png) = normalize_icon(&png) {
                    self.emit(Event::IconReady(domain, png));
                }
            }
        }
    }
    #[cfg(feature = "desktop")]
    fn copy(&mut self, id: &CredentialId) -> Result<(), YubiKeyError> {
        let c = self.credential(id)?;
        if c.valid_until.is_some_and(|t| t <= unix_time()) {
            return Err(YubiKeyError::Generic(
                "Code has expired; generate a fresh code".into(),
            ));
        }
        let text = c
            .code
            .as_ref()
            .ok_or_else(|| YubiKeyError::Generic("Generate a code first".into()))?
            .replace(' ', "");
        self.clipboard
            .copy(
                text,
                Duration::from_secs(u64::from(self.settings.clipboard_timeout_seconds)),
            )
            .map_err(YubiKeyError::Generic)?;
        Ok(())
    }
    #[cfg(feature = "desktop")]
    fn fetch_icon(&self, domain: &str) -> Result<(), YubiKeyError> {
        if !self.settings.allow_favicons {
            return Err(YubiKeyError::Generic(
                "Enable optional favicon downloads in Settings first".into(),
            ));
        }
        let domain = crate::settings::validated_domain(domain)
            .map_err(|e| YubiKeyError::Generic(e.to_string()))?;
        let mut url = url::Url::parse("https://www.google.com/s2/favicons")
            .map_err(|_| YubiKeyError::Generic("Invalid favicon service URL".into()))?;
        url.query_pairs_mut()
            .append_pair("domain", &domain)
            .append_pair("sz", "64");
        let response = ureq::get(url.as_str())
            .timeout(Duration::from_secs(10))
            .call()
            .map_err(|_| {
                YubiKeyError::Generic("Favicon download failed; check network access".into())
            })?;
        use std::io::Read;
        let mut bytes = Vec::new();
        response
            .into_reader()
            .take(256 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| YubiKeyError::Generic(e.to_string()))?;
        if bytes.len() > 256 * 1024 {
            return Err(YubiKeyError::Generic("Favicon exceeds size limit".into()));
        }
        let png = normalize_icon(&bytes)?;
        let dir = crate::settings::cache_dir()
            .map_err(|e| YubiKeyError::Generic(e.to_string()))?
            .join("favicons");
        std::fs::create_dir_all(&dir).map_err(|e| YubiKeyError::Generic(e.to_string()))?;
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new_in(&dir)
            .map_err(|e| YubiKeyError::Generic(e.to_string()))?;
        file.write_all(&png)
            .map_err(|e| YubiKeyError::Generic(e.to_string()))?;
        file.persist(dir.join(crate::settings::favicon_filename(&domain)))
            .map_err(|e| YubiKeyError::Generic(e.to_string()))?;
        self.load_icon(&domain);
        self.emit(Event::Success(Outcome::IconDownloaded));
        Ok(())
    }
}

fn normalize_icon(bytes: &[u8]) -> Result<Vec<u8>, YubiKeyError> {
    if bytes.len() > 256 * 1024 {
        return Err(YubiKeyError::Generic("Favicon exceeds size limit".into()));
    }
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| YubiKeyError::Generic(e.to_string()))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(1024);
    limits.max_image_height = Some(1024);
    limits.max_alloc = Some(4 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| YubiKeyError::Generic("Favicon is not a valid small image".into()))?
        .thumbnail(64, 64);
    let mut png = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| YubiKeyError::Generic(e.to_string()))?;
    Ok(png.into_inner())
}
