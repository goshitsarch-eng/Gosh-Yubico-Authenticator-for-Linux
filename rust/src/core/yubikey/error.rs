use thiserror::Error;

/// YubiKey OATH operation errors
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum YubiKeyError {
    #[error("No YubiKey found")]
    NoDevice,

    #[error("Failed to connect to YubiKey: {0}")]
    ConnectionFailed(String),

    #[error("Failed to establish PC/SC context: {0}")]
    ContextFailed(String),

    #[error("Failed to select OATH applet")]
    SelectFailed,

    #[error("Authentication required (YubiKey is password protected)")]
    AuthenticationRequired,

    #[error("Wrong password")]
    WrongPassword,

    #[error("Touch required on YubiKey")]
    TouchRequired,

    #[error("Credential not found: {0}")]
    CredentialNotFound(String),

    #[error("Credential already exists: {0}")]
    CredentialAlreadyExists(String),

    #[error("No space left on device")]
    NoSpace,

    #[error("Invalid credential name")]
    InvalidName,

    #[error("Invalid secret key: {0}")]
    InvalidSecret(String),

    #[error("APDU error: SW={sw1:02X}{sw2:02X}")]
    ApduError { sw1: u8, sw2: u8 },

    #[error("Invalid response from YubiKey: {0}")]
    InvalidResponse(String),

    #[error("PC/SC error: {0}")]
    PcscError(String),

    #[error("Communication timeout")]
    Timeout,

    #[error("Operation cancelled")]
    Cancelled,

    #[error("YubiKey disconnected")]
    Disconnected,

    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<pcsc::Error> for YubiKeyError {
    fn from(err: pcsc::Error) -> Self {
        match err {
            pcsc::Error::NoReadersAvailable => YubiKeyError::NoDevice,
            pcsc::Error::ReaderUnavailable => YubiKeyError::Disconnected,
            pcsc::Error::RemovedCard => YubiKeyError::Disconnected,
            pcsc::Error::Timeout => YubiKeyError::Timeout,
            pcsc::Error::Cancelled => YubiKeyError::Cancelled,
            _ => YubiKeyError::PcscError(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, YubiKeyError>;
