use crate::core::yubikey::{Algorithm, OathType};

/// Unique identifier for a credential (the raw name bytes from YubiKey)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CredentialId(pub Vec<u8>);

impl CredentialId {
    /// Get the credential name as a string
    pub fn as_str(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }
}

/// Represents an OATH credential stored on a YubiKey
#[derive(Debug, Clone)]
pub struct Credential {
    /// Unique identifier (raw name bytes)
    pub id: CredentialId,
    /// Service/website name (e.g., "Google", "GitHub")
    pub issuer: Option<String>,
    /// Account name (e.g., "user@example.com")
    pub account: String,
    /// TOTP or HOTP
    pub oath_type: OathType,
    /// Hash algorithm (SHA1, SHA256, SHA512)
    pub algorithm: Algorithm,
    /// Number of digits in the code (6, 7, or 8)
    pub digits: u8,
    /// Whether physical touch is required to generate code
    pub touch_required: bool,
    /// The current OTP code (if calculated)
    pub code: Option<String>,
    /// TOTP period in seconds (default: 30)
    pub period: u32,
}

impl Credential {
    /// Get a display name for the credential
    pub fn display_name(&self) -> String {
        match &self.issuer {
            Some(issuer) => format!("{}: {}", issuer, self.account),
            None => self.account.clone(),
        }
    }

    /// Check if this is a TOTP credential
    pub fn is_totp(&self) -> bool {
        self.oath_type == OathType::Totp
    }

    /// Check if this is an HOTP credential
    pub fn is_hotp(&self) -> bool {
        self.oath_type == OathType::Hotp
    }

    /// Set the code and return the formatted version
    pub fn set_code(&mut self, code: u32, digits: u8) {
        self.digits = digits;
        self.code = Some(format_code(code, digits));
    }

    /// Clear the code
    pub fn clear_code(&mut self) {
        self.code = None;
    }
}

/// Format an OTP code with a space in the middle for readability
pub fn format_code(code: u32, digits: u8) -> String {
    let code_str = format!("{:0width$}", code, width = digits as usize);
    match digits {
        6 => format!("{} {}", &code_str[..3], &code_str[3..]),
        8 => format!("{} {}", &code_str[..4], &code_str[4..]),
        7 => format!("{} {}", &code_str[..3], &code_str[3..]),
        _ => code_str,
    }
}

/// Parameters for adding a new credential
#[derive(Debug, Clone)]
pub struct NewCredential {
    pub issuer: Option<String>,
    pub account: String,
    pub secret: Vec<u8>,
    pub oath_type: OathType,
    pub algorithm: Algorithm,
    pub digits: u8,
    pub require_touch: bool,
    pub initial_counter: Option<u32>,
}

impl Default for NewCredential {
    fn default() -> Self {
        Self {
            issuer: None,
            account: String::new(),
            secret: Vec::new(),
            oath_type: OathType::Totp,
            algorithm: Algorithm::Sha1,
            digits: 6,
            require_touch: false,
            initial_counter: None,
        }
    }
}

/// Decode a base32-encoded secret key
pub fn decode_secret(secret: &str) -> Result<Vec<u8>, String> {
    // Remove spaces and dashes, convert to uppercase
    let cleaned: String = secret
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect::<String>()
        .to_uppercase();

    // Pad to multiple of 8 if needed
    let padded = if cleaned.len() % 8 != 0 {
        let padding_len = 8 - (cleaned.len() % 8);
        format!("{}{}", cleaned, "=".repeat(padding_len))
    } else {
        cleaned
    };

    data_encoding::BASE32
        .decode(padded.as_bytes())
        .map_err(|e| format!("Invalid base32: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_code() {
        assert_eq!(format_code(123456, 6), "123 456");
        assert_eq!(format_code(12345678, 8), "1234 5678");
        assert_eq!(format_code(1, 6), "000 001");
    }

    #[test]
    fn test_decode_secret() {
        // Standard test vector from RFC 4226
        let secret = decode_secret("GEZDGNBVGY3TQOJQ").unwrap();
        assert_eq!(secret, b"12345678901234567890");

        // With spaces
        let secret = decode_secret("GEZD GNBV GY3T QOJQ").unwrap();
        assert_eq!(secret, b"12345678901234567890");

        // Lowercase
        let secret = decode_secret("gezdgnbvgy3tqojq").unwrap();
        assert_eq!(secret, b"12345678901234567890");
    }
}
