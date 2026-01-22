use hmac::{Hmac, Mac};
use sha1::Sha1;
use getrandom::getrandom;

use super::apdu::{self, Algorithm, OathType, Tag, TlvParser};
use super::connection::YubiKeyConnection;
use super::error::{Result, YubiKeyError};
use crate::core::credential::{Credential, CredentialId};

type HmacSha1 = Hmac<Sha1>;

/// OATH operations on a YubiKey
pub struct OathSession {
    connection: YubiKeyConnection,
    authenticated: bool,
}

impl OathSession {
    /// Create a new OATH session from an existing connection
    pub fn new(connection: YubiKeyConnection) -> Self {
        Self {
            authenticated: !connection.requires_auth(),
            connection,
        }
    }

    /// Connect to a YubiKey and create an OATH session
    pub fn connect() -> Result<Self> {
        let connection = YubiKeyConnection::connect()?;
        Ok(Self::new(connection))
    }

    /// Check if authentication is required
    pub fn requires_auth(&self) -> bool {
        self.connection.requires_auth() && !self.authenticated
    }

    /// Authenticate with the YubiKey using a password
    pub fn validate(&mut self, password: &str) -> Result<()> {
        let challenge = self
            .connection
            .challenge
            .as_ref()
            .ok_or(YubiKeyError::Generic("No challenge available".into()))?;

        let algorithm = self
            .connection
            .challenge_algorithm
            .unwrap_or(Algorithm::Sha1);

        // Derive key from password using PBKDF2
        // The YubiKey expects: PBKDF2(password, device_id, 1000, key_length)
        let key = derive_key(password, &self.connection.device_id, algorithm);

        // Calculate HMAC of challenge
        let response = calculate_hmac(&key, challenge, algorithm);

        // Generate our own challenge for the device to respond to
        let our_challenge: [u8; 8] = rand_bytes()?;

        // Build and send VALIDATE command
        let apdu = apdu::build_validate_apdu(&response, &our_challenge);
        let response_data = self.connection.transmit(&apdu)?;
        let data = self.connection.check_status(&response_data)?;

        // Verify the device's response
        for tlv in TlvParser::new(data) {
            if tlv.tag == Tag::Response as u8 {
                let expected = calculate_hmac(&key, &our_challenge, algorithm);
                if tlv.value == expected {
                    self.authenticated = true;
                    return Ok(());
                }
            }
        }

        Err(YubiKeyError::WrongPassword)
    }

    /// List all credentials on the YubiKey
    pub fn list_credentials(&self) -> Result<Vec<Credential>> {
        if self.requires_auth() {
            return Err(YubiKeyError::AuthenticationRequired);
        }

        let apdu = apdu::build_list_apdu();
        let response = self.connection.transmit(&apdu)?;
        let data = self.connection.check_status(&response)?;

        let mut credentials = Vec::new();

        for tlv in TlvParser::new(data) {
            if tlv.tag == Tag::NameList as u8 && !tlv.value.is_empty() {
                let type_algo = tlv.value[0];
                let name = &tlv.value[1..];

                let oath_type = OathType::from_byte(type_algo).unwrap_or(OathType::Totp);
                let algorithm = Algorithm::from_byte(type_algo).unwrap_or(Algorithm::Sha1);

                let (issuer, account) = parse_credential_name(name);

                credentials.push(Credential {
                    id: CredentialId(name.to_vec()),
                    issuer,
                    account,
                    oath_type,
                    algorithm,
                    digits: 6, // Default, will be updated when calculating
                    touch_required: false, // Will be determined during calculate
                    code: None,
                    period: 30,
                });
            }
        }

        Ok(credentials)
    }

    /// Calculate codes for all TOTP credentials
    pub fn calculate_all(&self, timestamp: Option<u64>) -> Result<Vec<(CredentialId, Option<(u32, u8)>)>> {
        if self.requires_auth() {
            return Err(YubiKeyError::AuthenticationRequired);
        }

        let challenge = get_challenge(timestamp, 30);
        let apdu = apdu::build_calculate_all_apdu(&challenge);
        let response = self.connection.transmit(&apdu)?;
        let data = self.connection.check_status(&response)?;

        let mut results = Vec::new();
        let mut parser = TlvParser::new(data);

        while let Some(tlv) = parser.next() {
            if tlv.tag == Tag::Name as u8 {
                let name = tlv.value.to_vec();

                // Next TLV should be the response
                if let Some(resp_tlv) = parser.next() {
                    let code = match resp_tlv.tag {
                        t if t == Tag::TruncatedResponse as u8 => {
                            parse_truncated_response(resp_tlv.value)
                        }
                        t if t == Tag::Touch as u8 => {
                            // Touch required - no code available
                            None
                        }
                        t if t == Tag::Hotp as u8 => {
                            // HOTP - needs individual calculation
                            None
                        }
                        _ => None,
                    };

                    results.push((CredentialId(name), code));
                }
            }
        }

        Ok(results)
    }

    /// Calculate a single credential's code
    pub fn calculate(&self, credential: &Credential, timestamp: Option<u64>) -> Result<(u32, u8)> {
        if self.requires_auth() {
            return Err(YubiKeyError::AuthenticationRequired);
        }

        let period = credential.period;
        let challenge = match credential.oath_type {
            OathType::Totp => get_challenge(timestamp, period),
            OathType::Hotp => vec![], // HOTP uses stored counter
        };

        let apdu = apdu::build_calculate_apdu(&credential.id.0, &challenge);
        let response = self.connection.transmit(&apdu)?;

        // Check for touch required (timeout would indicate this)
        let data = match self.connection.check_status(&response) {
            Ok(d) => d,
            Err(YubiKeyError::ApduError { sw1: 0x69, sw2: 0x84 }) => {
                return Err(YubiKeyError::TouchRequired);
            }
            Err(e) => return Err(e),
        };

        for tlv in TlvParser::new(data) {
            if tlv.tag == Tag::TruncatedResponse as u8 || tlv.tag == Tag::Response as u8 {
                if let Some((code, digits)) = parse_truncated_response(tlv.value) {
                    return Ok((code, digits));
                }
            }
            if tlv.tag == Tag::Touch as u8 {
                return Err(YubiKeyError::TouchRequired);
            }
        }

        Err(YubiKeyError::InvalidResponse(
            "No code in response".into(),
        ))
    }

    /// Add a new credential to the YubiKey
    pub fn put_credential(
        &self,
        issuer: Option<&str>,
        account: &str,
        secret: &[u8],
        oath_type: OathType,
        algorithm: Algorithm,
        digits: u8,
        require_touch: bool,
        initial_counter: Option<u32>,
    ) -> Result<Credential> {
        if self.requires_auth() {
            return Err(YubiKeyError::AuthenticationRequired);
        }

        // Build credential name
        let name = build_credential_name(issuer, account);
        if name.is_empty() || name.len() > 64 {
            return Err(YubiKeyError::InvalidName);
        }

        // Validate and pad secret
        let padded_secret = pad_secret(secret, algorithm);

        let apdu = apdu::build_put_apdu(
            &name,
            &padded_secret,
            oath_type,
            algorithm,
            digits,
            require_touch,
            initial_counter,
        );

        let response = self.connection.transmit(&apdu)?;
        self.connection.check_status(&response)?;

        Ok(Credential {
            id: CredentialId(name.clone()),
            issuer: issuer.map(String::from),
            account: account.to_string(),
            oath_type,
            algorithm,
            digits,
            touch_required: require_touch,
            code: None,
            period: 30,
        })
    }

    /// Delete a credential from the YubiKey
    pub fn delete_credential(&self, credential: &Credential) -> Result<()> {
        if self.requires_auth() {
            return Err(YubiKeyError::AuthenticationRequired);
        }

        let apdu = apdu::build_delete_apdu(&credential.id.0);
        let response = self.connection.transmit(&apdu)?;
        self.connection.check_status(&response)?;

        Ok(())
    }

    /// Get OATH version
    pub fn version(&self) -> (u8, u8, u8) {
        self.connection.version
    }

    /// Get device ID
    pub fn device_id(&self) -> [u8; 8] {
        self.connection.device_id
    }

    /// Set or change the access code (password) for the OATH applet
    ///
    /// - If `new_password` is empty, removes password protection
    /// - If set, derives a key using PBKDF2 and stores it on the YubiKey
    pub fn set_code(&mut self, new_password: &str) -> Result<()> {
        if self.requires_auth() {
            return Err(YubiKeyError::AuthenticationRequired);
        }

        let device_id = self.connection.device_id;

        if new_password.is_empty() {
            // Remove password protection
            let apdu = apdu::build_set_code_apdu(&[], &[], &[]);
            let response = self.connection.transmit(&apdu)?;
            self.connection.check_status(&response)?;
            self.authenticated = true;
        } else {
            // Set new password
            let key = derive_key(new_password, &device_id, Algorithm::Sha1);

            // Generate a challenge for verification
            let challenge: [u8; 8] = rand_bytes()?;
            let response_hmac = calculate_hmac(&key, &challenge, Algorithm::Sha1);

            let apdu = apdu::build_set_code_apdu(&key, &challenge, &response_hmac);
            let response = self.connection.transmit(&apdu)?;
            self.connection.check_status(&response)?;

            // After setting password, we're authenticated with the new password
            self.authenticated = true;
        }

        Ok(())
    }

    /// Check if the OATH applet has password protection enabled
    pub fn has_password(&self) -> bool {
        self.connection.requires_auth()
    }
}

/// Parse credential name into (issuer, account)
fn parse_credential_name(name: &[u8]) -> (Option<String>, String) {
    let name_str = String::from_utf8_lossy(name);

    if let Some(colon_pos) = name_str.find(':') {
        let issuer = name_str[..colon_pos].to_string();
        let account = name_str[colon_pos + 1..].to_string();
        (Some(issuer), account)
    } else {
        (None, name_str.into_owned())
    }
}

/// Build credential name from issuer and account
fn build_credential_name(issuer: Option<&str>, account: &str) -> Vec<u8> {
    match issuer {
        Some(iss) if !iss.is_empty() => format!("{}:{}", iss, account).into_bytes(),
        _ => account.as_bytes().to_vec(),
    }
}

/// Get TOTP challenge for a given timestamp
fn get_challenge(timestamp: Option<u64>, period: u32) -> Vec<u8> {
    let period = if period == 0 { 30 } else { period };
    let ts = timestamp.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    let counter = ts / period as u64;
    counter.to_be_bytes().to_vec()
}

/// Parse truncated response to get code and digits
fn parse_truncated_response(value: &[u8]) -> Option<(u32, u8)> {
    if value.len() < 5 {
        return None;
    }

    let digits = value[0];
    if !(6..=8).contains(&digits) {
        return None;
    }
    let code = u32::from_be_bytes([value[1], value[2], value[3], value[4]]);

    // Truncate to specified digits
    let divisor = match digits {
        6 => 1_000_000,
        7 => 10_000_000,
        8 => 100_000_000,
        _ => return None,
    };
    let truncated = code % divisor;

    Some((truncated, digits))
}

/// Derive authentication key from password
fn derive_key(password: &str, salt: &[u8], algorithm: Algorithm) -> Vec<u8> {
    let key_len = algorithm.hmac_key_size();
    let mut key = vec![0u8; key_len];

    pbkdf2::pbkdf2_hmac::<Sha1>(password.as_bytes(), salt, 1000, &mut key);

    key
}

/// Calculate HMAC for authentication
fn calculate_hmac(key: &[u8], data: &[u8], _algorithm: Algorithm) -> Vec<u8> {
    // YubiKey always uses SHA-1 for authentication, regardless of the algorithm field
    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Pad secret to match algorithm requirements
fn pad_secret(secret: &[u8], algorithm: Algorithm) -> Vec<u8> {
    let required_len = algorithm.hmac_key_size();
    if secret.len() >= required_len {
        secret.to_vec()
    } else {
        let mut padded = secret.to_vec();
        padded.resize(required_len, 0);
        padded
    }
}

/// Generate random bytes (simple implementation)
fn rand_bytes<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0u8; N];
    getrandom(&mut bytes)
        .map_err(|e| YubiKeyError::Generic(format!("Random generation failed: {}", e)))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_credential_name() {
        assert_eq!(
            parse_credential_name(b"Google:user@example.com"),
            (Some("Google".to_string()), "user@example.com".to_string())
        );

        assert_eq!(
            parse_credential_name(b"user@example.com"),
            (None, "user@example.com".to_string())
        );
    }

    #[test]
    fn test_build_credential_name() {
        assert_eq!(
            build_credential_name(Some("Google"), "user@example.com"),
            b"Google:user@example.com".to_vec()
        );

        assert_eq!(
            build_credential_name(None, "user@example.com"),
            b"user@example.com".to_vec()
        );
    }

    #[test]
    fn test_parse_truncated_response() {
        // 6 digits, code = 123456
        let response = [0x06, 0x00, 0x01, 0xE2, 0x40]; // 123456 = 0x1E240
        let result = parse_truncated_response(&response);
        assert_eq!(result, Some((123456, 6)));
    }
}
