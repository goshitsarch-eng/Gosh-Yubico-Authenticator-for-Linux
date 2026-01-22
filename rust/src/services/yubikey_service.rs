use std::collections::HashMap;
use std::thread;

use async_channel::{Receiver, Sender};

use crate::core::credential::{Credential, CredentialId, NewCredential};
use crate::core::yubikey::{OathSession, YubiKeyError};

/// Commands sent from UI to the YubiKey worker
#[derive(Debug)]
pub enum Command {
    /// Connect to YubiKey and refresh credentials
    Connect,
    /// Authenticate with password
    Authenticate(String),
    /// Refresh credentials list and calculate all TOTP codes
    RefreshCredentials,
    /// Calculate a single credential (for touch-required or HOTP)
    CalculateCredential(CredentialId),
    /// Add a new credential
    AddCredential(NewCredential),
    /// Delete a credential
    DeleteCredential(CredentialId),
    /// Change or set the OATH password (empty string removes password)
    SetPassword(String),
    /// Disconnect and clean up
    Disconnect,
    /// Shutdown the worker
    Shutdown,
}

/// Events sent from the YubiKey worker to the UI
#[derive(Debug, Clone)]
pub enum Event {
    /// Successfully connected to YubiKey
    Connected {
        version: (u8, u8, u8),
        device_id: [u8; 8],
    },
    /// YubiKey disconnected
    Disconnected,
    /// Authentication required
    AuthenticationRequired,
    /// Authentication successful
    AuthenticationSuccessful,
    /// Authentication failed
    AuthenticationFailed(String),
    /// Credentials list updated
    CredentialsUpdated(Vec<Credential>),
    /// A single credential was calculated
    CredentialCalculated {
        id: CredentialId,
        code: String,
        digits: u8,
    },
    /// Touch required on YubiKey
    TouchRequired(CredentialId),
    /// Credential added successfully
    CredentialAdded(Credential),
    /// Credential deleted successfully
    CredentialDeleted(CredentialId),
    /// Password changed successfully
    PasswordChanged,
    /// Password removed successfully
    PasswordRemoved,
    /// Error occurred
    Error(String),
    /// Worker is ready
    Ready,
}

/// YubiKey background service
#[derive(Debug)]
pub struct YubiKeyService {
    command_tx: Sender<Command>,
    event_rx: Receiver<Event>,
}

impl YubiKeyService {
    /// Create a new YubiKey service and start the background worker
    pub fn new() -> Self {
        let (command_tx, command_rx) = async_channel::bounded::<Command>(16);
        let (event_tx, event_rx) = async_channel::bounded::<Event>(16);

        // Spawn worker thread
        thread::spawn(move || {
            Worker::new(command_rx, event_tx).run();
        });

        Self {
            command_tx,
            event_rx,
        }
    }

    /// Send a command to the worker
    pub fn send(&self, command: Command) {
        if let Err(e) = self.command_tx.send_blocking(command) {
            log::error!("Failed to send command: {}", e);
        }
    }

    /// Get the event receiver for the UI to listen on
    pub fn event_receiver(&self) -> Receiver<Event> {
        self.event_rx.clone()
    }

    /// Request connection to YubiKey
    pub fn connect(&self) {
        self.send(Command::Connect);
    }

    /// Authenticate with password
    pub fn authenticate(&self, password: String) {
        self.send(Command::Authenticate(password));
    }

    /// Refresh credentials
    pub fn refresh(&self) {
        self.send(Command::RefreshCredentials);
    }

    /// Calculate a specific credential
    pub fn calculate(&self, id: CredentialId) {
        self.send(Command::CalculateCredential(id));
    }

    /// Add a credential
    pub fn add_credential(&self, credential: NewCredential) {
        self.send(Command::AddCredential(credential));
    }

    /// Delete a credential
    pub fn delete_credential(&self, id: CredentialId) {
        self.send(Command::DeleteCredential(id));
    }

    /// Set or change the OATH password (empty string removes password)
    pub fn set_password(&self, password: String) {
        self.send(Command::SetPassword(password));
    }

    /// Shutdown the service
    pub fn shutdown(&self) {
        let _ = self.command_tx.send_blocking(Command::Shutdown);
    }
}

impl Default for YubiKeyService {
    fn default() -> Self {
        Self::new()
    }
}

/// Background worker that handles YubiKey communication
struct Worker {
    command_rx: Receiver<Command>,
    event_tx: Sender<Event>,
    session: Option<OathSession>,
}

impl Worker {
    fn new(command_rx: Receiver<Command>, event_tx: Sender<Event>) -> Self {
        Self {
            command_rx,
            event_tx,
            session: None,
        }
    }

    fn run(mut self) {
        log::info!("YubiKey worker started");
        self.send_event(Event::Ready);

        loop {
            match self.command_rx.recv_blocking() {
                Ok(Command::Shutdown) => {
                    log::info!("YubiKey worker shutting down");
                    break;
                }
                Ok(command) => {
                    self.handle_command(command);
                }
                Err(e) => {
                    log::error!("Command channel closed: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::Connect => self.handle_connect(),
            Command::Authenticate(password) => self.handle_authenticate(&password),
            Command::RefreshCredentials => self.handle_refresh(),
            Command::CalculateCredential(id) => self.handle_calculate(id),
            Command::AddCredential(cred) => self.handle_add_credential(cred),
            Command::DeleteCredential(id) => self.handle_delete_credential(id),
            Command::SetPassword(password) => self.handle_set_password(&password),
            Command::Disconnect => self.handle_disconnect(),
            Command::Shutdown => unreachable!(),
        }
    }

    fn handle_connect(&mut self) {
        log::info!("Connecting to YubiKey...");

        match OathSession::connect() {
            Ok(session) => {
                let version = session.version();
                let device_id = session.device_id();
                let requires_auth = session.requires_auth();

                self.session = Some(session);

                self.send_event(Event::Connected { version, device_id });

                if requires_auth {
                    self.send_event(Event::AuthenticationRequired);
                } else {
                    // Auto-refresh credentials
                    self.handle_refresh();
                }
            }
            Err(YubiKeyError::NoDevice) => {
                self.send_event(Event::Disconnected);
            }
            Err(e) => {
                self.send_event(Event::Error(e.to_string()));
            }
        }
    }

    fn handle_authenticate(&mut self, password: &str) {
        let Some(ref mut session) = self.session else {
            self.send_event(Event::Error("No YubiKey connected".into()));
            return;
        };

        match session.validate(password) {
            Ok(()) => {
                self.send_event(Event::AuthenticationSuccessful);
                self.handle_refresh();
            }
            Err(YubiKeyError::WrongPassword) => {
                self.send_event(Event::AuthenticationFailed("Wrong password".into()));
            }
            Err(e) => {
                self.send_event(Event::Error(e.to_string()));
            }
        }
    }

    fn handle_refresh(&mut self) {
        let Some(ref session) = self.session else {
            self.send_event(Event::Disconnected);
            return;
        };

        // List credentials
        let mut credentials = match session.list_credentials() {
            Ok(creds) => creds,
            Err(YubiKeyError::AuthenticationRequired) => {
                self.send_event(Event::AuthenticationRequired);
                return;
            }
            Err(e) => {
                self.send_event(Event::Error(e.to_string()));
                return;
            }
        };

        // Calculate all TOTP credentials at once
        match session.calculate_all(None) {
            Ok(results) => {
                let mut index = HashMap::with_capacity(credentials.len());
                for (idx, cred) in credentials.iter().enumerate() {
                    index.insert(cred.id.clone(), idx);
                }

                for (id, code_info) in results {
                    if let Some(idx) = index.get(&id).copied() {
                        let cred = &mut credentials[idx];
                        match code_info {
                            Some((code, digits)) => {
                                cred.set_code(code, digits);
                            }
                            None => {
                                // Touch required or HOTP
                                cred.touch_required = true;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to calculate all: {}", e);
            }
        }

        self.send_event(Event::CredentialsUpdated(credentials));
    }

    fn handle_calculate(&mut self, id: CredentialId) {
        let Some(ref session) = self.session else {
            self.send_event(Event::Disconnected);
            return;
        };

        // Find the credential in our list
        let credentials = match session.list_credentials() {
            Ok(creds) => creds,
            Err(e) => {
                self.send_event(Event::Error(e.to_string()));
                return;
            }
        };

        let Some(credential) = credentials.iter().find(|c| c.id == id) else {
            self.send_event(Event::Error("Credential not found".into()));
            return;
        };

        match session.calculate(credential, None) {
            Ok((code, digits)) => {
                let formatted = crate::core::credential::format_code(code, digits);
                self.send_event(Event::CredentialCalculated {
                    id,
                    code: formatted,
                    digits,
                });
            }
            Err(YubiKeyError::TouchRequired) => {
                self.send_event(Event::TouchRequired(id));
            }
            Err(e) => {
                self.send_event(Event::Error(e.to_string()));
            }
        }
    }

    fn handle_add_credential(&mut self, new_cred: NewCredential) {
        let Some(ref session) = self.session else {
            self.send_event(Event::Disconnected);
            return;
        };

        match session.put_credential(
            new_cred.issuer.as_deref(),
            &new_cred.account,
            &new_cred.secret,
            new_cred.oath_type,
            new_cred.algorithm,
            new_cred.digits,
            new_cred.require_touch,
            new_cred.initial_counter,
        ) {
            Ok(credential) => {
                self.send_event(Event::CredentialAdded(credential));
                // Refresh to show the new credential with code
                self.handle_refresh();
            }
            Err(e) => {
                self.send_event(Event::Error(e.to_string()));
            }
        }
    }

    fn handle_delete_credential(&mut self, id: CredentialId) {
        let Some(ref session) = self.session else {
            self.send_event(Event::Disconnected);
            return;
        };

        // Create a temporary credential struct for deletion
        let credential = Credential {
            id: id.clone(),
            issuer: None,
            account: String::new(),
            oath_type: crate::core::yubikey::OathType::Totp,
            algorithm: crate::core::yubikey::Algorithm::Sha1,
            digits: 6,
            touch_required: false,
            code: None,
            period: 30,
        };

        match session.delete_credential(&credential) {
            Ok(()) => {
                self.send_event(Event::CredentialDeleted(id));
                // Refresh to update the list
                self.handle_refresh();
            }
            Err(e) => {
                self.send_event(Event::Error(e.to_string()));
            }
        }
    }

    fn handle_set_password(&mut self, new_password: &str) {
        let Some(ref mut session) = self.session else {
            self.send_event(Event::Error("No YubiKey connected".into()));
            return;
        };

        match session.set_code(new_password) {
            Ok(()) => {
                if new_password.is_empty() {
                    self.send_event(Event::PasswordRemoved);
                } else {
                    self.send_event(Event::PasswordChanged);
                }
            }
            Err(YubiKeyError::AuthenticationRequired) => {
                self.send_event(Event::AuthenticationRequired);
            }
            Err(e) => {
                self.send_event(Event::Error(e.to_string()));
            }
        }
    }

    fn handle_disconnect(&mut self) {
        self.session = None;
        self.send_event(Event::Disconnected);
    }

    fn send_event(&self, event: Event) {
        if let Err(e) = self.event_tx.send_blocking(event) {
            log::error!("Failed to send event: {}", e);
        }
    }
}
