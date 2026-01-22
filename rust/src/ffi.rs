use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::{Arc, Mutex, Once};
use std::thread;

use crate::core::credential::{decode_secret, Credential, CredentialId, NewCredential};
use crate::core::yubikey::{Algorithm, OathType};
use crate::services::{Event, YubiKeyService};

pub type GoshEventCallback = Option<unsafe extern "C" fn(*const GoshEvent, *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GoshVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GoshCredential {
    pub id_ptr: *const u8,
    pub id_len: usize,
    pub issuer: *const c_char,
    pub account: *const c_char,
    pub code: *const c_char,
    pub oath_type: u8,
    pub algorithm: u8,
    pub digits: u8,
    pub touch_required: u8,
    pub period: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GoshEvent {
    pub event_type: u32,
    pub version: GoshVersion,
    pub device_id: [u8; 8],
    pub message: *const c_char,
    pub credentials: *const GoshCredential,
    pub credentials_len: usize,
    pub id_ptr: *const u8,
    pub id_len: usize,
    pub code: *const c_char,
    pub digits: u8,
}

#[repr(C)]
struct GoshEventBox {
    pub event: GoshEvent,
    strings: Vec<CString>,
    credential_strings: Vec<CString>,
    credential_ids: Vec<Vec<u8>>,
    credentials: Vec<GoshCredential>,
    id_bytes: Option<Vec<u8>>,
}

#[derive(Clone, Copy)]
struct CallbackState {
    callback: GoshEventCallback,
    user_data: usize,
}

pub struct GoshClient {
    service: YubiKeyService,
    callback: Arc<Mutex<Option<CallbackState>>>,
    last_error: Mutex<Option<CString>>,
}

const EVENT_READY: u32 = 0;
const EVENT_CONNECTED: u32 = 1;
const EVENT_DISCONNECTED: u32 = 2;
const EVENT_AUTH_REQUIRED: u32 = 3;
const EVENT_AUTH_SUCCESS: u32 = 4;
const EVENT_AUTH_FAILED: u32 = 5;
const EVENT_CREDENTIALS_UPDATED: u32 = 6;
const EVENT_CREDENTIAL_CALCULATED: u32 = 7;
const EVENT_TOUCH_REQUIRED: u32 = 8;
const EVENT_CREDENTIAL_ADDED: u32 = 9;
const EVENT_CREDENTIAL_DELETED: u32 = 10;
const EVENT_PASSWORD_CHANGED: u32 = 12;
const EVENT_PASSWORD_REMOVED: u32 = 13;
const EVENT_ERROR: u32 = 11;

#[no_mangle]
pub extern "C" fn gosh_init_logging() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .is_test(false)
            .try_init();
    });
}

#[no_mangle]
pub extern "C" fn gosh_client_new() -> *mut GoshClient {
    let client = Box::new(GoshClient::new());
    Box::into_raw(client)
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_free(client: *mut GoshClient) {
    if client.is_null() {
        return;
    }
    let boxed = Box::from_raw(client);
    boxed.service.shutdown();
    // Drop happens here
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_set_callback(
    client: *mut GoshClient,
    callback: GoshEventCallback,
    user_data: *mut c_void,
) {
    if let Some(client) = client.as_ref() {
        let mut guard = client.callback.lock().unwrap();
        *guard = Some(CallbackState {
            callback,
            user_data: user_data as usize,
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_connect(client: *mut GoshClient) {
    if let Some(client) = client.as_ref() {
        client.service.connect();
    }
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_refresh(client: *mut GoshClient) {
    if let Some(client) = client.as_ref() {
        client.service.refresh();
    }
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_authenticate(client: *mut GoshClient, password: *const c_char) {
    if let Some(client) = client.as_ref() {
        if password.is_null() {
            return;
        }
        let password = CStr::from_ptr(password).to_string_lossy().into_owned();
        client.service.authenticate(password);
    }
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_calculate(
    client: *mut GoshClient,
    id_ptr: *const u8,
    id_len: usize,
) {
    if let Some(client) = client.as_ref() {
        if id_ptr.is_null() || id_len == 0 {
            return;
        }
        let id = std::slice::from_raw_parts(id_ptr, id_len).to_vec();
        client.service.calculate(CredentialId(id));
    }
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_delete_credential(
    client: *mut GoshClient,
    id_ptr: *const u8,
    id_len: usize,
) {
    if let Some(client) = client.as_ref() {
        if id_ptr.is_null() || id_len == 0 {
            return;
        }
        let id = std::slice::from_raw_parts(id_ptr, id_len).to_vec();
        client.service.delete_credential(CredentialId(id));
    }
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_add_credential(
    client: *mut GoshClient,
    issuer: *const c_char,
    account: *const c_char,
    secret: *const c_char,
    oath_type: u8,
    algorithm: u8,
    digits: u8,
    require_touch: u8,
    initial_counter: u32,
    has_initial_counter: u8,
) -> bool {
    let Some(client) = client.as_ref() else {
        return false;
    };

    let account = if account.is_null() {
        client.set_last_error("Account is required");
        return false;
    } else {
        CStr::from_ptr(account).to_string_lossy().into_owned()
    };

    if account.trim().is_empty() {
        client.set_last_error("Account is required");
        return false;
    }

    let secret = if secret.is_null() {
        client.set_last_error("Secret is required");
        return false;
    } else {
        CStr::from_ptr(secret).to_string_lossy().into_owned()
    };

    let secret_bytes = match decode_secret(&secret) {
        Ok(bytes) => bytes,
        Err(e) => {
            client.set_last_error(&e);
            return false;
        }
    };

    let issuer = if issuer.is_null() {
        None
    } else {
        let value = CStr::from_ptr(issuer).to_string_lossy().into_owned();
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    };

    let Some(oath_type) = oath_type_from_u8(oath_type) else {
        client.set_last_error("Invalid OATH type");
        return false;
    };

    let Some(algorithm) = algorithm_from_u8(algorithm) else {
        client.set_last_error("Invalid algorithm");
        return false;
    };

    let initial_counter = if has_initial_counter != 0 {
        Some(initial_counter)
    } else {
        None
    };

    let new_credential = NewCredential {
        issuer,
        account,
        secret: secret_bytes,
        oath_type,
        algorithm,
        digits,
        require_touch: require_touch != 0,
        initial_counter,
    };

    client.service.add_credential(new_credential);
    true
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_set_password(
    client: *mut GoshClient,
    password: *const c_char,
) {
    if let Some(client) = client.as_ref() {
        // null or empty string means remove password
        let password = if password.is_null() {
            String::new()
        } else {
            CStr::from_ptr(password).to_string_lossy().into_owned()
        };
        client.service.set_password(password);
    }
}

#[no_mangle]
pub unsafe extern "C" fn gosh_client_last_error_take(client: *mut GoshClient) -> *mut c_char {
    if let Some(client) = client.as_ref() {
        if let Some(err) = client.last_error.lock().unwrap().take() {
            return err.into_raw();
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn gosh_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }
    let _ = CString::from_raw(value);
}

#[no_mangle]
pub unsafe extern "C" fn gosh_event_free(event: *mut GoshEvent) {
    if event.is_null() {
        return;
    }
    let _ = Box::from_raw(event as *mut GoshEventBox);
}

impl GoshClient {
    fn new() -> Self {
        let service = YubiKeyService::new();
        let callback = Arc::new(Mutex::new(None));
        let event_rx = service.event_receiver();
        let callback_clone = Arc::clone(&callback);

        thread::spawn(move || {
            while let Ok(event) = event_rx.recv_blocking() {
                let state: Option<CallbackState> = *callback_clone.lock().unwrap();
                if let Some(state) = state {
                    if let Some(cb) = state.callback {
                        let event_box = build_event_box(event);
                        let raw = Box::into_raw(event_box);
                        let event_ptr = raw as *const GoshEvent;
                        unsafe {
                            cb(event_ptr, state.user_data as *mut c_void);
                        }
                        // ownership transferred to callback via gosh_event_free
                        let _ = raw;
                    }
                }
            }
        });

        Self {
            service,
            callback,
            last_error: Mutex::new(None),
        }
    }

    fn set_last_error(&self, message: &str) {
        let mut guard = self.last_error.lock().unwrap();
        *guard = Some(CString::new(message).unwrap_or_else(|_| CString::new("Invalid error").unwrap()));
    }
}

fn oath_type_from_u8(value: u8) -> Option<OathType> {
    match value {
        0x10 => Some(OathType::Hotp),
        0x20 => Some(OathType::Totp),
        _ => None,
    }
}

fn algorithm_from_u8(value: u8) -> Option<Algorithm> {
    match value {
        0x01 => Some(Algorithm::Sha1),
        0x02 => Some(Algorithm::Sha256),
        0x03 => Some(Algorithm::Sha512),
        _ => None,
    }
}

fn build_event_box(event: Event) -> Box<GoshEventBox> {
    let mut event_box = GoshEventBox {
        event: GoshEvent {
            event_type: EVENT_READY,
            version: GoshVersion {
                major: 0,
                minor: 0,
                patch: 0,
            },
            device_id: [0; 8],
            message: ptr::null(),
            credentials: ptr::null(),
            credentials_len: 0,
            id_ptr: ptr::null(),
            id_len: 0,
            code: ptr::null(),
            digits: 0,
        },
        strings: Vec::new(),
        credential_strings: Vec::new(),
        credential_ids: Vec::new(),
        credentials: Vec::new(),
        id_bytes: None,
    };

    match event {
        Event::Ready => {
            event_box.event.event_type = EVENT_READY;
        }
        Event::Connected { version, device_id } => {
            event_box.event.event_type = EVENT_CONNECTED;
            event_box.event.version = GoshVersion {
                major: version.0,
                minor: version.1,
                patch: version.2,
            };
            event_box.event.device_id = device_id;
        }
        Event::Disconnected => {
            event_box.event.event_type = EVENT_DISCONNECTED;
        }
        Event::AuthenticationRequired => {
            event_box.event.event_type = EVENT_AUTH_REQUIRED;
        }
        Event::AuthenticationSuccessful => {
            event_box.event.event_type = EVENT_AUTH_SUCCESS;
        }
        Event::AuthenticationFailed(msg) => {
            event_box.event.event_type = EVENT_AUTH_FAILED;
            push_message(&mut event_box, msg);
        }
        Event::CredentialsUpdated(credentials) => {
            event_box.event.event_type = EVENT_CREDENTIALS_UPDATED;
            fill_credentials(&mut event_box, &credentials);
        }
        Event::CredentialCalculated { id, code, digits } => {
            event_box.event.event_type = EVENT_CREDENTIAL_CALCULATED;
            set_id(&mut event_box, &id);
            push_code(&mut event_box, code);
            event_box.event.digits = digits;
        }
        Event::TouchRequired(id) => {
            event_box.event.event_type = EVENT_TOUCH_REQUIRED;
            set_id(&mut event_box, &id);
        }
        Event::CredentialAdded(credential) => {
            event_box.event.event_type = EVENT_CREDENTIAL_ADDED;
            fill_credentials(&mut event_box, &[credential]);
        }
        Event::CredentialDeleted(id) => {
            event_box.event.event_type = EVENT_CREDENTIAL_DELETED;
            set_id(&mut event_box, &id);
        }
        Event::PasswordChanged => {
            event_box.event.event_type = EVENT_PASSWORD_CHANGED;
        }
        Event::PasswordRemoved => {
            event_box.event.event_type = EVENT_PASSWORD_REMOVED;
        }
        Event::Error(msg) => {
            event_box.event.event_type = EVENT_ERROR;
            push_message(&mut event_box, msg);
        }
    }

    Box::new(event_box)
}

fn push_message(event_box: &mut GoshEventBox, message: String) {
    let cstr = CString::new(message).unwrap_or_else(|_| CString::new("Invalid message").unwrap());
    event_box.event.message = cstr.as_ptr();
    event_box.strings.push(cstr);
}

fn push_code(event_box: &mut GoshEventBox, code: String) {
    let cstr = CString::new(code).unwrap_or_else(|_| CString::new("Invalid code").unwrap());
    event_box.event.code = cstr.as_ptr();
    event_box.strings.push(cstr);
}

fn set_id(event_box: &mut GoshEventBox, id: &CredentialId) {
    event_box.id_bytes = Some(id.0.clone());
    if let Some(bytes) = event_box.id_bytes.as_ref() {
        event_box.event.id_ptr = bytes.as_ptr();
        event_box.event.id_len = bytes.len();
    }
}

fn fill_credentials(event_box: &mut GoshEventBox, credentials: &[Credential]) {
    for credential in credentials {
        let id = credential.id.0.clone();
        let issuer = credential
            .issuer
            .clone()
            .unwrap_or_else(|| String::new());
        let account = credential.account.clone();
        let code = credential.code.clone().unwrap_or_else(|| String::new());

        let issuer_c = CString::new(issuer).unwrap_or_else(|_| CString::new("").unwrap());
        let account_c = CString::new(account).unwrap_or_else(|_| CString::new("").unwrap());
        let code_c = CString::new(code).unwrap_or_else(|_| CString::new("").unwrap());

        event_box.credential_ids.push(id);
        let id_index = event_box.credential_ids.len() - 1;
        let id_ref = &event_box.credential_ids[id_index];

        event_box.credential_strings.push(issuer_c);
        event_box.credential_strings.push(account_c);
        event_box.credential_strings.push(code_c);

        let issuer_ptr = event_box
            .credential_strings
            .get(event_box.credential_strings.len() - 3)
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null());
        let account_ptr = event_box
            .credential_strings
            .get(event_box.credential_strings.len() - 2)
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null());
        let code_ptr = event_box
            .credential_strings
            .get(event_box.credential_strings.len() - 1)
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null());

        let cred = GoshCredential {
            id_ptr: id_ref.as_ptr(),
            id_len: id_ref.len(),
            issuer: if credential.issuer.is_some() {
                issuer_ptr
            } else {
                ptr::null()
            },
            account: account_ptr,
            code: if credential.code.is_some() {
                code_ptr
            } else {
                ptr::null()
            },
            oath_type: credential.oath_type as u8,
            algorithm: credential.algorithm as u8,
            digits: credential.digits,
            touch_required: if credential.touch_required { 1 } else { 0 },
            period: credential.period,
        };

        event_box.credentials.push(cred);
    }

    event_box.event.credentials_len = event_box.credentials.len();
    event_box.event.credentials = if event_box.credentials.is_empty() {
        ptr::null()
    } else {
        event_box.credentials.as_ptr()
    };
}
