use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Subscription, Task, Theme};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::core::credential::{Credential, CredentialId};
use crate::services::clipboard;
use crate::services::settings::AppSettings;
use crate::services::yubikey_service::{ServiceCommand, ServiceEvent, YubiKeyService};
use crate::ui::screens;

/// Current screen/page of the application
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    AddCredential,
    Settings,
    KeyInfo,
    About,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// Authentication state
#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    NotRequired,
    Required,
    Authenticating,
    Authenticated,
    Failed(String),
}

/// The main application state
pub struct App {
    // Navigation
    pub screen: Screen,

    // YubiKey state
    pub service: YubiKeyService,
    pub connection_state: ConnectionState,
    pub auth_state: AuthState,
    pub credentials: Vec<Credential>,
    pub yubikey_version: Option<(u8, u8, u8)>,
    pub yubikey_device_id: Option<[u8; 8]>,
    pub yubikey_has_password: bool,

    // UI state
    pub search_query: String,
    pub password_input: String,
    pub show_touch_prompt: bool,
    pub touch_credential_id: Option<CredentialId>,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub totp_remaining: u32,

    // Settings
    pub settings: AppSettings,

    // Add credential form
    pub add_form: AddCredentialForm,

    // Settings form
    pub new_password: String,
    pub confirm_password: String,

    // Confirm delete
    pub delete_confirm: Option<CredentialId>,
}

/// Form state for adding a credential
#[derive(Debug, Clone, Default)]
pub struct AddCredentialForm {
    pub issuer: String,
    pub account: String,
    pub secret: String,
    pub oath_type_index: usize, // 0=TOTP, 1=HOTP
    pub algorithm_index: usize, // 0=SHA1, 1=SHA256, 2=SHA512
    pub digits_index: usize,    // 0=6, 1=7, 2=8
    pub require_touch: bool,
    pub initial_counter: String,
    pub uri_input: String,
    pub error: Option<String>,
}

/// All messages the application can handle
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    // Navigation
    NavigateTo(Screen),

    // YubiKey events
    ServiceEvent(ServiceEvent),
    ConnectYubiKey,
    RefreshCredentials,
    Disconnect,

    // Authentication
    PasswordInputChanged(String),
    SubmitPassword,

    // Credentials
    SearchChanged(String),
    CopyCode(String),
    CalculateCredential(CredentialId),
    DeleteCredential(CredentialId),
    ConfirmDelete(CredentialId),
    CancelDelete,

    // Add credential form
    AddFormIssuerChanged(String),
    AddFormAccountChanged(String),
    AddFormSecretChanged(String),
    AddFormOathTypeChanged(usize),
    AddFormAlgorithmChanged(usize),
    AddFormDigitsChanged(usize),
    AddFormTouchToggled(bool),
    AddFormCounterChanged(String),
    AddFormUriChanged(String),
    AddFormParseUri,
    AddFormSubmit,
    AddFormScanQr,

    // Settings
    ThemeModeChanged(String),
    ClipboardTimeoutChanged(String),
    NewPasswordChanged(String),
    ConfirmPasswordChanged(String),
    SubmitNewPassword,
    RemovePassword,

    // Timer
    Tick,

    // Status
    DismissStatus,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let settings = AppSettings::load();
        let service = YubiKeyService::new();

        let app = Self {
            screen: Screen::Home,
            service,
            connection_state: ConnectionState::Disconnected,
            auth_state: AuthState::NotRequired,
            credentials: Vec::new(),
            yubikey_version: None,
            yubikey_device_id: None,
            yubikey_has_password: false,
            search_query: String::new(),
            password_input: String::new(),
            show_touch_prompt: false,
            touch_credential_id: None,
            status_message: None,
            totp_remaining: 30,
            settings,
            add_form: AddCredentialForm::default(),
            new_password: String::new(),
            confirm_password: String::new(),
            delete_confirm: None,
        };

        (app, Task::none())
    }

    pub fn theme(&self) -> Theme {
        match self.settings.theme_mode.as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::default(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            // 1-second timer for TOTP countdown and polling events
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        ])
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::NavigateTo(screen) => {
                if screen == Screen::AddCredential {
                    self.add_form = AddCredentialForm::default();
                }
                self.screen = screen;
            }

            // Connect
            Message::ConnectYubiKey => {
                self.connection_state = ConnectionState::Connecting;
                self.service.connect();
            }
            Message::RefreshCredentials => {
                self.service.refresh();
            }
            Message::Disconnect => {
                self.service.send(ServiceCommand::Disconnect);
                self.connection_state = ConnectionState::Disconnected;
                self.auth_state = AuthState::NotRequired;
                self.credentials.clear();
                self.yubikey_version = None;
                self.yubikey_device_id = None;
            }

            // Auth
            Message::PasswordInputChanged(val) => {
                self.password_input = val;
            }
            Message::SubmitPassword => {
                if !self.password_input.is_empty() {
                    self.auth_state = AuthState::Authenticating;
                    self.service.authenticate(self.password_input.clone());
                    self.password_input.clear();
                }
            }

            // Search
            Message::SearchChanged(query) => {
                self.search_query = query;
            }

            // Credential actions
            Message::CopyCode(code) => {
                let clean_code = code.replace(' ', "");
                let timeout = if self.settings.clipboard_timeout > 0 {
                    Some(self.settings.clipboard_timeout)
                } else {
                    None
                };
                clipboard::copy_to_clipboard(&clean_code, timeout);
                self.status_message = Some(("Code copied to clipboard".into(), false));
            }
            Message::CalculateCredential(id) => {
                self.service.calculate(id);
            }
            Message::ConfirmDelete(id) => {
                self.delete_confirm = Some(id);
            }
            Message::DeleteCredential(id) => {
                self.service.delete_credential(id);
                self.delete_confirm = None;
            }
            Message::CancelDelete => {
                self.delete_confirm = None;
            }

            // Add credential form
            Message::AddFormIssuerChanged(val) => self.add_form.issuer = val,
            Message::AddFormAccountChanged(val) => self.add_form.account = val,
            Message::AddFormSecretChanged(val) => self.add_form.secret = val,
            Message::AddFormOathTypeChanged(idx) => self.add_form.oath_type_index = idx,
            Message::AddFormAlgorithmChanged(idx) => self.add_form.algorithm_index = idx,
            Message::AddFormDigitsChanged(idx) => self.add_form.digits_index = idx,
            Message::AddFormTouchToggled(val) => self.add_form.require_touch = val,
            Message::AddFormCounterChanged(val) => self.add_form.initial_counter = val,
            Message::AddFormUriChanged(val) => self.add_form.uri_input = val,
            Message::AddFormParseUri => {
                self.parse_uri_into_form();
            }
            Message::AddFormSubmit => {
                self.submit_add_credential();
            }
            Message::AddFormScanQr => {
                self.scan_qr_code();
            }

            // Settings
            Message::ThemeModeChanged(mode) => {
                self.settings.theme_mode = mode;
                self.settings.save();
            }
            Message::ClipboardTimeoutChanged(val) => {
                if let Ok(timeout) = val.parse::<u64>() {
                    self.settings.clipboard_timeout = timeout;
                    self.settings.save();
                }
            }
            Message::NewPasswordChanged(val) => self.new_password = val,
            Message::ConfirmPasswordChanged(val) => self.confirm_password = val,
            Message::SubmitNewPassword => {
                if self.new_password == self.confirm_password && !self.new_password.is_empty() {
                    self.service.set_password(self.new_password.clone());
                    self.new_password.clear();
                    self.confirm_password.clear();
                } else if self.new_password != self.confirm_password {
                    self.status_message = Some(("Passwords do not match".into(), true));
                }
            }
            Message::RemovePassword => {
                self.service.set_password(String::new());
            }

            // Timer tick - update TOTP countdown and poll for service events
            Message::Tick => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                self.totp_remaining = (30 - (now % 30)) as u32;

                // Auto-refresh at period boundary
                if self.totp_remaining == 30
                    && self.connection_state == ConnectionState::Connected
                    && self.auth_state != AuthState::Required
                {
                    self.service.refresh();
                }

                // Poll for service events
                while let Some(event) = self.service.try_recv() {
                    self.handle_service_event(event);
                }
            }

            Message::DismissStatus => {
                self.status_message = None;
            }

            // Service events received via subscription
            Message::ServiceEvent(event) => {
                self.handle_service_event(event);
            }
        }

        Task::none()
    }

    fn handle_service_event(&mut self, event: ServiceEvent) {
        match event {
            ServiceEvent::Ready => {
                log::info!("Service ready");
            }
            ServiceEvent::Connected {
                version,
                device_id,
                has_password,
            } => {
                self.connection_state = ConnectionState::Connected;
                self.yubikey_version = Some(version);
                self.yubikey_device_id = Some(device_id);
                self.yubikey_has_password = has_password;
            }
            ServiceEvent::Disconnected => {
                self.connection_state = ConnectionState::Disconnected;
                self.auth_state = AuthState::NotRequired;
                self.credentials.clear();
                self.yubikey_version = None;
                self.yubikey_device_id = None;
            }
            ServiceEvent::AuthenticationRequired => {
                self.auth_state = AuthState::Required;
            }
            ServiceEvent::AuthenticationSuccessful => {
                self.auth_state = AuthState::Authenticated;
                self.status_message = Some(("Authentication successful".into(), false));
            }
            ServiceEvent::AuthenticationFailed(msg) => {
                self.auth_state = AuthState::Failed(msg);
            }
            ServiceEvent::CredentialsUpdated(creds) => {
                self.credentials = creds;
            }
            ServiceEvent::CredentialCalculated { id, code, digits: _ } => {
                if let Some(cred) = self.credentials.iter_mut().find(|c| c.id == id) {
                    cred.code = Some(code);
                    cred.touch_required = false;
                }
                self.show_touch_prompt = false;
                self.touch_credential_id = None;
            }
            ServiceEvent::TouchRequired(id) => {
                self.show_touch_prompt = true;
                self.touch_credential_id = Some(id);
            }
            ServiceEvent::CredentialAdded(_) => {
                self.screen = Screen::Home;
                self.status_message = Some(("Credential added successfully".into(), false));
            }
            ServiceEvent::CredentialDeleted(_) => {
                self.status_message = Some(("Credential deleted".into(), false));
            }
            ServiceEvent::PasswordChanged => {
                self.yubikey_has_password = true;
                self.status_message = Some(("Password changed successfully".into(), false));
            }
            ServiceEvent::PasswordRemoved => {
                self.yubikey_has_password = false;
                self.status_message = Some(("Password removed".into(), false));
            }
            ServiceEvent::Error(msg) => {
                self.status_message = Some((msg, true));
            }
        }
    }

    fn parse_uri_into_form(&mut self) {
        match crate::core::credential::parse_otpauth_uri(&self.add_form.uri_input) {
            Ok(cred) => {
                self.add_form.issuer = cred.issuer.unwrap_or_default();
                self.add_form.account = cred.account;
                self.add_form.secret = data_encoding::BASE32_NOPAD.encode(&cred.secret);
                self.add_form.oath_type_index = match cred.oath_type {
                    crate::core::yubikey::OathType::Totp => 0,
                    crate::core::yubikey::OathType::Hotp => 1,
                };
                self.add_form.algorithm_index = match cred.algorithm {
                    crate::core::yubikey::Algorithm::Sha1 => 0,
                    crate::core::yubikey::Algorithm::Sha256 => 1,
                    crate::core::yubikey::Algorithm::Sha512 => 2,
                };
                self.add_form.digits_index = match cred.digits {
                    7 => 1,
                    8 => 2,
                    _ => 0,
                };
                if let Some(counter) = cred.initial_counter {
                    self.add_form.initial_counter = counter.to_string();
                }
                self.add_form.error = None;
                self.add_form.uri_input.clear();
            }
            Err(e) => {
                self.add_form.error = Some(e);
            }
        }
    }

    fn submit_add_credential(&mut self) {
        if self.add_form.account.is_empty() {
            self.add_form.error = Some("Account name is required".into());
            return;
        }
        if self.add_form.secret.is_empty() {
            self.add_form.error = Some("Secret key is required".into());
            return;
        }

        let secret = match crate::core::credential::decode_secret(&self.add_form.secret) {
            Ok(s) => s,
            Err(e) => {
                self.add_form.error = Some(format!("Invalid secret: {}", e));
                return;
            }
        };

        let oath_type = match self.add_form.oath_type_index {
            1 => crate::core::yubikey::OathType::Hotp,
            _ => crate::core::yubikey::OathType::Totp,
        };

        let algorithm = match self.add_form.algorithm_index {
            1 => crate::core::yubikey::Algorithm::Sha256,
            2 => crate::core::yubikey::Algorithm::Sha512,
            _ => crate::core::yubikey::Algorithm::Sha1,
        };

        let digits = match self.add_form.digits_index {
            1 => 7,
            2 => 8,
            _ => 6,
        };

        let initial_counter = if oath_type == crate::core::yubikey::OathType::Hotp {
            self.add_form.initial_counter.parse().ok()
        } else {
            None
        };

        let new_cred = crate::core::credential::NewCredential {
            issuer: if self.add_form.issuer.is_empty() {
                None
            } else {
                Some(self.add_form.issuer.clone())
            },
            account: self.add_form.account.clone(),
            secret,
            oath_type,
            algorithm,
            digits,
            require_touch: self.add_form.require_touch,
            initial_counter,
        };

        self.service.add_credential(new_cred);
    }

    fn scan_qr_code(&mut self) {
        // Open file dialog to pick a QR code image
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
            .set_title("Select QR Code Image")
            .pick_file()
        {
            match image::open(&path) {
                Ok(img) => {
                    let gray = img.to_luma8();
                    let mut prepared = rqrr::PreparedImage::prepare(gray);
                    let grids = prepared.detect_grids();

                    if let Some(grid) = grids.into_iter().next() {
                        match grid.decode() {
                            Ok((_meta, content)) => {
                                self.add_form.uri_input = content;
                                self.parse_uri_into_form();
                            }
                            Err(e) => {
                                self.add_form.error =
                                    Some(format!("Failed to decode QR code: {}", e));
                            }
                        }
                    } else {
                        self.add_form.error = Some("No QR code found in image".into());
                    }
                }
                Err(e) => {
                    self.add_form.error = Some(format!("Failed to open image: {}", e));
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content: Element<Message> = match self.screen {
            Screen::Home => screens::home::view(self),
            Screen::AddCredential => screens::add_credential::view(self),
            Screen::Settings => screens::settings::view(self),
            Screen::KeyInfo => screens::key_info::view(self),
            Screen::About => screens::about::view(self),
        };

        // Wrap with status bar if there's a message
        if let Some((ref msg, is_error)) = self.status_message {
            let status_color = if is_error {
                crate::ui::theme::Palette::ERROR
            } else {
                crate::ui::theme::Palette::SUCCESS
            };

            let status_bar = container(
                row![
                    text(msg).color(iced::Color::WHITE).size(14),
                    Space::with_width(Length::Fill),
                    button(text("x").size(12))
                        .on_press(Message::DismissStatus)
                        .padding(4)
                        .style(button::text),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding(8)
            .width(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(status_color)),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                text_color: Some(iced::Color::WHITE),
            });

            column![status_bar, content].into()
        } else if self.show_touch_prompt {
            // Touch prompt overlay
            let touch_bar = container(
                row![
                    text("Touch your YubiKey...").size(14),
                    Space::with_width(Length::Fill),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding(12)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(
                    crate::ui::theme::Palette::WARNING,
                )),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                text_color: Some(iced::Color::WHITE),
            });

            column![touch_bar, content].into()
        } else {
            content
        }
    }

    /// Get filtered credentials based on search query
    pub fn filtered_credentials(&self) -> Vec<&Credential> {
        if self.search_query.is_empty() {
            self.credentials.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.credentials
                .iter()
                .filter(|c| {
                    c.account.to_lowercase().contains(&query)
                        || c.issuer
                            .as_ref()
                            .map(|i| i.to_lowercase().contains(&query))
                            .unwrap_or(false)
                })
                .collect()
        }
    }
}
