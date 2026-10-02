use super::apdu::{self, Algorithm, OathType, Tag, TlvParser};
use super::connection::{response_data, OathConnection, YubiKeyConnection};
use super::error::{Result, YubiKeyError};
use crate::core::credential::{Credential, CredentialId, NewCredential};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use zeroize::Zeroizing;

pub struct OathSession {
    connection: Box<dyn OathConnection>,
    authenticated: bool,
    password_set: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Calculation {
    Code(u32, u8),
    Touch,
    Hotp,
}
pub type CalculatedCredentials = Vec<(CredentialId, Calculation)>;
impl OathSession {
    pub fn new(connection: impl OathConnection + 'static) -> Self {
        let password_set = connection.challenge().is_some();
        Self {
            connection: Box::new(connection),
            authenticated: !password_set,
            password_set,
        }
    }
    pub fn connect() -> Result<Self> {
        Ok(Self::new(YubiKeyConnection::connect()?))
    }
    pub fn requires_auth(&self) -> bool {
        !self.authenticated
    }
    pub fn has_password(&self) -> bool {
        self.password_set
    }
    pub fn is_present(&self) -> Result<bool> {
        self.connection.is_present()
    }
    pub fn version(&self) -> (u8, u8, u8) {
        self.connection.version()
    }
    pub fn device_id(&self) -> [u8; 8] {
        self.connection.device_id()
    }
    pub fn validate(&mut self, password: &str) -> Result<()> {
        let challenge = self
            .connection
            .challenge()
            .ok_or(YubiKeyError::AuthenticationRequired)?;
        let key = derive_key(password, &self.device_id());
        let response = hmac_bytes(&key, challenge)?;
        let mut ours = [0; 8];
        getrandom::getrandom(&mut ours)
            .map_err(|_| YubiKeyError::Generic("Secure random generation failed".into()))?;
        let raw = self
            .connection
            .transmit(&apdu::build_validate_apdu(&response, &ours))?;
        let data = response_data(&raw).map_err(|error| match error {
            YubiKeyError::AuthenticationRequired => YubiKeyError::WrongPassword,
            other => other,
        })?;
        let tlvs = parse_tlvs(data)?;
        let proof = tlvs
            .iter()
            .find(|t| t.tag == Tag::Response as u8)
            .ok_or_else(|| invalid("Missing mutual-authentication proof"))?;
        let mut verifier = Hmac::<Sha1>::new_from_slice(&key)
            .map_err(|_| invalid("Invalid authentication key"))?;
        verifier.update(&ours);
        verifier
            .verify_slice(proof.value)
            .map_err(|_| YubiKeyError::WrongPassword)?;
        self.authenticated = true;
        Ok(())
    }
    pub fn list_credentials(&self) -> Result<Vec<Credential>> {
        self.ensure_unlocked()?;
        let raw = self.connection.transmit(&apdu::build_list_apdu())?;
        parse_tlvs(response_data(&raw)?)?
            .into_iter()
            .map(|tlv| {
                if tlv.tag != Tag::NameList as u8 || tlv.value.len() < 2 {
                    return Err(invalid("Invalid credential list entry"));
                }
                let kind = OathType::from_byte(tlv.value[0])
                    .ok_or_else(|| invalid("Unsupported OATH type"))?;
                let algorithm = Algorithm::from_byte(tlv.value[0])
                    .ok_or_else(|| invalid("Unsupported OATH algorithm"))?;
                let name = &tlv.value[1..];
                let (issuer, account, period) = parse_name(name, kind)?;
                Ok(Credential {
                    id: CredentialId(name.to_vec()),
                    issuer,
                    account,
                    oath_type: kind,
                    algorithm,
                    digits: 6,
                    touch_required: false,
                    code: None,
                    period,
                    valid_until: None,
                })
            })
            .collect()
    }
    pub fn calculate_all(&self, timestamp: Option<u64>) -> Result<CalculatedCredentials> {
        self.ensure_unlocked()?;
        let raw = self
            .connection
            .transmit(&apdu::build_calculate_all_apdu(&challenge(timestamp, 30)))?;
        let data = response_data(&raw).map_err(|error| match error {
            YubiKeyError::AuthenticationRequired => YubiKeyError::WrongPassword,
            other => other,
        })?;
        let tlvs = parse_tlvs(data)?;
        if !tlvs.len().is_multiple_of(2) {
            return Err(invalid("Unpaired calculate-all response"));
        }
        tlvs.as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                if pair[0].tag != Tag::Name as u8 || pair[0].value.is_empty() {
                    return Err(invalid("Missing calculation credential name"));
                }
                let value = match Tag::from_byte(pair[1].tag) {
                    Some(Tag::TruncatedResponse) => {
                        let (code, digits) = parse_code(pair[1].value)?;
                        Calculation::Code(code, digits)
                    }
                    Some(Tag::Touch) => Calculation::Touch,
                    Some(Tag::Hotp) => Calculation::Hotp,
                    _ => return Err(invalid("Unexpected calculation response")),
                };
                Ok((CredentialId(pair[0].value.to_vec()), value))
            })
            .collect()
    }
    pub fn calculate(&self, credential: &Credential, timestamp: Option<u64>) -> Result<(u32, u8)> {
        self.ensure_unlocked()?;
        let time = if credential.is_totp() {
            challenge(timestamp, credential.period)
        } else {
            Vec::new()
        };
        let raw = self
            .connection
            .transmit(&apdu::build_calculate_apdu(&credential.id.0, &time))?;
        for tlv in parse_tlvs(response_data(&raw)?)? {
            if tlv.tag == Tag::TruncatedResponse as u8 {
                return parse_code(tlv.value);
            }
            if tlv.tag == Tag::Touch as u8 {
                return Err(YubiKeyError::TouchRequired);
            }
        }
        Err(invalid("Missing OTP response"))
    }
    pub fn put(&self, new: &NewCredential) -> Result<Credential> {
        self.ensure_unlocked()?;
        new.validate().map_err(YubiKeyError::Generic)?;
        let name = new.name();
        if self.list_credentials()?.iter().any(|c| c.id.0 == name) {
            return Err(YubiKeyError::CredentialAlreadyExists);
        }
        let secret = prepare_secret(&new.secret, new.algorithm);
        let command = Zeroizing::new(apdu::build_put_apdu(
            &name,
            &secret,
            new.oath_type,
            new.algorithm,
            new.digits,
            new.require_touch,
            new.initial_counter,
        ));
        let raw = self.connection.transmit(&command)?;
        response_data(&raw)?;
        Ok(Credential {
            id: CredentialId(name),
            issuer: new.issuer.clone(),
            account: new.account.clone(),
            oath_type: new.oath_type,
            algorithm: new.algorithm,
            digits: new.digits,
            touch_required: new.require_touch,
            code: None,
            period: new.period,
            valid_until: None,
        })
    }
    pub fn delete_credential(&self, credential: &Credential) -> Result<()> {
        self.ensure_unlocked()?;
        if credential.id.0.is_empty() || credential.id.0.len() > 64 {
            return Err(YubiKeyError::InvalidName);
        }
        let raw = self
            .connection
            .transmit(&apdu::build_delete_apdu(&credential.id.0))?;
        response_data(&raw)?;
        Ok(())
    }
    pub fn set_code(&mut self, password: &str) -> Result<()> {
        self.ensure_unlocked()?;
        let command = if password.is_empty() {
            apdu::build_set_code_apdu(&[], &[], &[])
        } else {
            let key = derive_key(password, &self.device_id());
            let mut nonce = [0; 8];
            getrandom::getrandom(&mut nonce)
                .map_err(|_| invalid("Secure random generation failed"))?;
            apdu::build_set_code_apdu(&key, &nonce, &hmac_bytes(&key, &nonce)?)
        };
        let command = Zeroizing::new(command);
        let raw = self.connection.transmit(&command)?;
        response_data(&raw)?;
        self.password_set = !password.is_empty();
        Ok(())
    }
    fn ensure_unlocked(&self) -> Result<()> {
        if self.authenticated {
            Ok(())
        } else {
            Err(YubiKeyError::AuthenticationRequired)
        }
    }
}
fn invalid(text: &str) -> YubiKeyError {
    YubiKeyError::InvalidResponse(text.into())
}
pub fn parse_tlvs(data: &[u8]) -> Result<Vec<apdu::Tlv<'_>>> {
    let mut parser = TlvParser::new(data);
    let entries = parser.by_ref().collect();
    if !parser.is_empty() {
        return Err(invalid("Malformed TLV data"));
    }
    Ok(entries)
}
pub fn parse_name(name: &[u8], kind: OathType) -> Result<(Option<String>, String, u32)> {
    let mut label =
        std::str::from_utf8(name).map_err(|_| invalid("Credential name is not valid UTF-8"))?;
    let mut period = 30;
    if kind == OathType::Totp {
        if let Some((prefix, rest)) = label.split_once('/') {
            if prefix.chars().all(|c| c.is_ascii_digit()) {
                period = prefix.parse().map_err(|_| invalid("Invalid TOTP period"))?;
                if !(1..=86400).contains(&period) {
                    return Err(invalid("Invalid TOTP period"));
                }
                label = rest;
            }
        }
    }
    let (issuer, account) = match label.split_once(':') {
        Some((i, a)) => (Some(i.to_string()), a.to_string()),
        None => (None, label.to_string()),
    };
    if account.is_empty() {
        return Err(invalid("Empty account name"));
    }
    Ok((issuer, account, period))
}
fn challenge(timestamp: Option<u64>, period: u32) -> Vec<u8> {
    let ts = timestamp.unwrap_or_else(crate::app::unix_time);
    (ts / u64::from(period.max(1))).to_be_bytes().to_vec()
}
fn parse_code(value: &[u8]) -> Result<(u32, u8)> {
    if value.len() != 5 || !(6..=8).contains(&value[0]) {
        return Err(invalid("Invalid truncated OTP response"));
    }
    let code = u32::from_be_bytes([value[1], value[2], value[3], value[4]]) & 0x7fff_ffff;
    Ok((code % 10u32.pow(u32::from(value[0])), value[0]))
}
/// Matches yubikit.oath: PBKDF2-HMAC-SHA1, 1000 iterations, 16 bytes.
fn derive_key(password: &str, salt: &[u8]) -> Zeroizing<Vec<u8>> {
    let mut key = Zeroizing::new(vec![0; 16]);
    pbkdf2::pbkdf2_hmac::<Sha1>(password.as_bytes(), salt, 1000, &mut key);
    key
}
fn hmac_bytes(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let mut mac = Hmac::<Sha1>::new_from_slice(key).map_err(|_| invalid("Invalid HMAC key"))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}
fn prepare_secret(secret: &[u8], algo: Algorithm) -> Zeroizing<Vec<u8>> {
    let block = if algo == Algorithm::Sha512 { 128 } else { 64 };
    let mut value = if secret.len() > block {
        match algo {
            Algorithm::Sha1 => Sha1::digest(secret).to_vec(),
            Algorithm::Sha256 => Sha256::digest(secret).to_vec(),
            Algorithm::Sha512 => Sha512::digest(secret).to_vec(),
        }
    } else {
        secret.to_vec()
    };
    if value.len() < 14 {
        value.resize(14, 0);
    }
    Zeroizing::new(value)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nondefault_period_name_roundtrip() {
        assert_eq!(
            parse_name(b"60/Example:alice", OathType::Totp).unwrap(),
            (Some("Example".into()), "alice".into(), 60)
        );
    }
    #[test]
    fn malformed_tlv_fails_closed() {
        assert!(parse_tlvs(&[0x71, 3, 1]).is_err());
        assert!(parse_tlvs(&[0x71]).is_err());
    }
    #[test]
    fn password_key_matches_yubico() {
        assert_eq!(derive_key("password", b"12345678").as_slice(), &hex());
    }
    fn hex() -> Vec<u8> {
        data_encoding::HEXLOWER
            .decode(b"f531154d46d1bdbbcc1fcce02d6b4c93")
            .unwrap()
    }
    #[test]
    fn hotp_and_touch_are_distinct() {
        assert_ne!(Calculation::Touch, Calculation::Hotp);
    }
    #[test]
    fn secret_shortening_follows_hmac() {
        assert_eq!(
            prepare_secret(&[7; 80], Algorithm::Sha256).as_slice(),
            Sha256::digest([7; 80]).as_slice()
        );
    }
}
