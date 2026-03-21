use std::collections::HashMap;
use std::thread;

use tokio::sync::mpsc;

use crate::core::credential::{Credential, CredentialId, NewCredential};
use crate::core::yubikey::{OathSession, OathType, Algorithm, YubiKeyError};

/// Commands sent from UI to the YubiKey worker
#[derive(Debug)]
pub enum ServiceCommand {
    Connect,
    Authenticate(String),
    RefreshCredentials,
    CalculateCredential(CredentialId),
    AddCredential(NewCredential),
    DeleteCredential(CredentialId),
    SetPassword(String),
    Disconnect,
    Shutdown,
}

/// Events sent from the YubiKey worker to the UI
#[derive(Debug, Clone)]
pub enum ServiceEvent {
    Connected {
        version: (u8, u8, u8),
        device_id: [u8; 8],
        has_password: bool,
    },
    Disconnected,
    AuthenticationRequired,
    AuthenticationSuccessful,
    AuthenticationFailed(String),
    CredentialsUpdated(Vec<Credential>),
    CredentialCalculated {
        id: CredentialId,
        code: String,
        digits: u8,
    },
    TouchRequired(CredentialId),
    CredentialAdded(Credential),
    CredentialDeleted(CredentialId),
    PasswordChanged,
    PasswordRemoved,
    Error(String),
    Ready,
}

/// YubiKey background service
pub struct YubiKeyService {
    command_tx: mpsc::UnboundedSender<ServiceCommand>,
    event_rx: mpsc::UnboundedReceiver<ServiceEvent>,
}

impl YubiKeyService {
    /// Create a new YubiKey service and start the background worker
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel::<ServiceCommand>();
        let (event_tx, event_rx) = mpsc::unbounded_channel::<ServiceEvent>();

        thread::spawn(move || {
            Worker::new(command_rx, event_tx).run();
        });

        Self {
            command_tx,
            event_rx,
        }
    }

    /// Send a command to the worker
    pub fn send(&self, command: ServiceCommand) {
        if let Err(e) = self.command_tx.send(command) {
            log::error!("Failed to send command: {}", e);
        }
    }

    /// Try to receive an event (non-blocking)
    pub fn try_recv(&mut self) -> Option<ServiceEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Get a mutable reference to the event receiver for async polling
    pub fn event_receiver_mut(&mut self) -> &mut mpsc::UnboundedReceiver<ServiceEvent> {
        &mut self.event_rx
    }

    pub fn connect(&self) {
        self.send(ServiceCommand::Connect);
    }

    pub fn authenticate(&self, password: String) {
        self.send(ServiceCommand::Authenticate(password));
    }

    pub fn refresh(&self) {
        self.send(ServiceCommand::RefreshCredentials);
    }

    pub fn calculate(&self, id: CredentialId) {
        self.send(ServiceCommand::CalculateCredential(id));
    }

    pub fn add_credential(&self, credential: NewCredential) {
        self.send(ServiceCommand::AddCredential(credential));
    }

    pub fn delete_credential(&self, id: CredentialId) {
        self.send(ServiceCommand::DeleteCredential(id));
    }

    pub fn set_password(&self, password: String) {
        self.send(ServiceCommand::SetPassword(password));
    }

    pub fn shutdown(&self) {
        let _ = self.command_tx.send(ServiceCommand::Shutdown);
    }
}

/// Background worker that handles YubiKey communication
struct Worker {
    command_rx: mpsc::UnboundedReceiver<ServiceCommand>,
    event_tx: mpsc::UnboundedSender<ServiceEvent>,
    session: Option<OathSession>,
}

impl Worker {
    fn new(
        command_rx: mpsc::UnboundedReceiver<ServiceCommand>,
        event_tx: mpsc::UnboundedSender<ServiceEvent>,
    ) -> Self {
        Self {
            command_rx,
            event_tx,
            session: None,
        }
    }

    fn run(mut self) {
        log::info!("YubiKey worker started");
        self.send_event(ServiceEvent::Ready);

        loop {
            match self.command_rx.blocking_recv() {
                Some(ServiceCommand::Shutdown) => {
                    log::info!("YubiKey worker shutting down");
                    break;
                }
                Some(command) => {
                    self.handle_command(command);
                }
                None => {
                    log::info!("Command channel closed");
                    break;
                }
            }
        }
    }

    fn handle_command(&mut self, command: ServiceCommand) {
        match command {
            ServiceCommand::Connect => self.handle_connect(),
            ServiceCommand::Authenticate(password) => self.handle_authenticate(&password),
            ServiceCommand::RefreshCredentials => self.handle_refresh(),
            ServiceCommand::CalculateCredential(id) => self.handle_calculate(id),
            ServiceCommand::AddCredential(cred) => self.handle_add_credential(cred),
            ServiceCommand::DeleteCredential(id) => self.handle_delete_credential(id),
            ServiceCommand::SetPassword(password) => self.handle_set_password(&password),
            ServiceCommand::Disconnect => self.handle_disconnect(),
            ServiceCommand::Shutdown => unreachable!(),
        }
    }

    fn handle_connect(&mut self) {
        log::info!("Connecting to YubiKey...");

        match OathSession::connect() {
            Ok(session) => {
                let version = session.version();
                let device_id = session.device_id();
                let requires_auth = session.requires_auth();
                let has_password = session.has_password();

                self.session = Some(session);

                self.send_event(ServiceEvent::Connected {
                    version,
                    device_id,
                    has_password,
                });

                if requires_auth {
                    self.send_event(ServiceEvent::AuthenticationRequired);
                } else {
                    self.handle_refresh();
                }
            }
            Err(YubiKeyError::NoDevice) => {
                self.send_event(ServiceEvent::Disconnected);
            }
            Err(e) => {
                self.send_event(ServiceEvent::Error(e.to_string()));
            }
        }
    }

    fn handle_authenticate(&mut self, password: &str) {
        let Some(ref mut session) = self.session else {
            self.send_event(ServiceEvent::Error("No YubiKey connected".into()));
            return;
        };

        match session.validate(password) {
            Ok(()) => {
                self.send_event(ServiceEvent::AuthenticationSuccessful);
                self.handle_refresh();
            }
            Err(YubiKeyError::WrongPassword) => {
                self.send_event(ServiceEvent::AuthenticationFailed("Wrong password".into()));
            }
            Err(e) => {
                self.send_event(ServiceEvent::Error(e.to_string()));
            }
        }
    }

    fn handle_refresh(&mut self) {
        let Some(ref session) = self.session else {
            self.send_event(ServiceEvent::Disconnected);
            return;
        };

        let mut credentials = match session.list_credentials() {
            Ok(creds) => creds,
            Err(YubiKeyError::AuthenticationRequired) => {
                self.send_event(ServiceEvent::AuthenticationRequired);
                return;
            }
            Err(e) => {
                self.send_event(ServiceEvent::Error(e.to_string()));
                return;
            }
        };

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

        self.send_event(ServiceEvent::CredentialsUpdated(credentials));
    }

    fn handle_calculate(&mut self, id: CredentialId) {
        let Some(ref session) = self.session else {
            self.send_event(ServiceEvent::Disconnected);
            return;
        };

        let credentials = match session.list_credentials() {
            Ok(creds) => creds,
            Err(e) => {
                self.send_event(ServiceEvent::Error(e.to_string()));
                return;
            }
        };

        let Some(credential) = credentials.iter().find(|c| c.id == id) else {
            self.send_event(ServiceEvent::Error("Credential not found".into()));
            return;
        };

        match session.calculate(credential, None) {
            Ok((code, digits)) => {
                let formatted = crate::core::credential::format_code(code, digits);
                self.send_event(ServiceEvent::CredentialCalculated {
                    id,
                    code: formatted,
                    digits,
                });
            }
            Err(YubiKeyError::TouchRequired) => {
                self.send_event(ServiceEvent::TouchRequired(id));
            }
            Err(e) => {
                self.send_event(ServiceEvent::Error(e.to_string()));
            }
        }
    }

    fn handle_add_credential(&mut self, new_cred: NewCredential) {
        let Some(ref session) = self.session else {
            self.send_event(ServiceEvent::Disconnected);
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
                self.send_event(ServiceEvent::CredentialAdded(credential));
                self.handle_refresh();
            }
            Err(e) => {
                self.send_event(ServiceEvent::Error(e.to_string()));
            }
        }
    }

    fn handle_delete_credential(&mut self, id: CredentialId) {
        let Some(ref session) = self.session else {
            self.send_event(ServiceEvent::Disconnected);
            return;
        };

        let credential = Credential {
            id: id.clone(),
            issuer: None,
            account: String::new(),
            oath_type: OathType::Totp,
            algorithm: Algorithm::Sha1,
            digits: 6,
            touch_required: false,
            code: None,
            period: 30,
        };

        match session.delete_credential(&credential) {
            Ok(()) => {
                self.send_event(ServiceEvent::CredentialDeleted(id));
                self.handle_refresh();
            }
            Err(e) => {
                self.send_event(ServiceEvent::Error(e.to_string()));
            }
        }
    }

    fn handle_set_password(&mut self, new_password: &str) {
        let Some(ref mut session) = self.session else {
            self.send_event(ServiceEvent::Error("No YubiKey connected".into()));
            return;
        };

        match session.set_code(new_password) {
            Ok(()) => {
                if new_password.is_empty() {
                    self.send_event(ServiceEvent::PasswordRemoved);
                } else {
                    self.send_event(ServiceEvent::PasswordChanged);
                }
            }
            Err(YubiKeyError::AuthenticationRequired) => {
                self.send_event(ServiceEvent::AuthenticationRequired);
            }
            Err(e) => {
                self.send_event(ServiceEvent::Error(e.to_string()));
            }
        }
    }

    fn handle_disconnect(&mut self) {
        self.session = None;
        self.send_event(ServiceEvent::Disconnected);
    }

    fn send_event(&self, event: ServiceEvent) {
        if let Err(e) = self.event_tx.send(event) {
            log::error!("Failed to send event: {}", e);
        }
    }
}
