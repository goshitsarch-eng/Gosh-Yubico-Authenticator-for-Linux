use pcsc::{Card, Context, Protocols, Scope, ShareMode, MAX_ATR_SIZE, MAX_BUFFER_SIZE};

use super::apdu::{self, sw, Tag, TlvParser};
use super::error::{Result, YubiKeyError};

/// Connection to a YubiKey via PC/SC
pub struct YubiKeyConnection {
    card: std::cell::RefCell<Card>,
    /// OATH applet version (major, minor, patch)
    pub version: (u8, u8, u8),
    /// Device ID (8 bytes)
    pub device_id: [u8; 8],
    /// Challenge for validation (if password protected)
    pub challenge: Option<Vec<u8>>,
    /// Algorithm used for validation
    pub challenge_algorithm: Option<apdu::Algorithm>,
}

type SelectOathResponse = (
    (u8, u8, u8),
    [u8; 8],
    Option<Vec<u8>>,
    Option<apdu::Algorithm>,
);

impl YubiKeyConnection {
    /// Establish a connection to the first available YubiKey with OATH applet
    pub fn connect() -> Result<Self> {
        let ctx = Context::establish(Scope::User)?;

        // Get list of readers
        let mut readers_buf = [0u8; 2048];
        let readers = ctx.list_readers(&mut readers_buf)?;

        // Try to connect to each reader and select OATH applet
        for reader in readers {
            let reader_name = reader.to_string_lossy();

            // Skip non-YubiKey readers (basic heuristic)
            // YubiKey readers typically contain "Yubi" or "YubiKey" in the name
            // But we also try generic readers in case the naming is different
            log::debug!("Trying reader: {}", reader_name);

            match ctx.connect(reader, ShareMode::Shared, Protocols::ANY) {
                Ok(card) => {
                    // Try to select OATH applet
                    match Self::select_oath_applet(&card) {
                        Ok((version, device_id, challenge, algo)) => {
                            log::info!(
                                "Connected to YubiKey with OATH {}.{}.{}",
                                version.0,
                                version.1,
                                version.2
                            );
                            return Ok(Self {
                                card: std::cell::RefCell::new(card),
                                version,
                                device_id,
                                challenge,
                                challenge_algorithm: algo,
                            });
                        }
                        Err(e) => {
                            log::debug!("Failed to select OATH on {}: {}", reader_name, e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    log::debug!("Failed to connect to {}: {}", reader_name, e);
                    continue;
                }
            }
        }

        Err(YubiKeyError::NoDevice)
    }

    /// Select the OATH applet and parse the response
    fn select_oath_applet(card: &Card) -> Result<SelectOathResponse> {
        let select_apdu = apdu::build_select_apdu();
        let mut recv_buf = [0u8; MAX_BUFFER_SIZE];

        let response = card.transmit(&select_apdu, &mut recv_buf)?;

        Self::parse_select_response(response)
    }

    pub(crate) fn parse_select_response(response: &[u8]) -> Result<SelectOathResponse> {
        if response.len() < 2 {
            return Err(YubiKeyError::InvalidResponse("Response too short".into()));
        }

        let sw1 = response[response.len() - 2];
        let sw2 = response[response.len() - 1];

        if (sw1, sw2) != sw::SUCCESS {
            return Err(YubiKeyError::SelectFailed);
        }

        // Parse SELECT response
        let data = &response[..response.len() - 2];
        let mut version = (0u8, 0u8, 0u8);
        let mut device_id = None;
        let mut challenge = None;
        let mut algo = None;

        let mut parser = TlvParser::new(data);
        for tlv in parser.by_ref() {
            match Tag::from_byte(tlv.tag) {
                Some(Tag::Version) if tlv.value.len() >= 3 => {
                    version = (tlv.value[0], tlv.value[1], tlv.value[2]);
                }
                Some(Tag::Name) if tlv.value.len() == 8 => {
                    if device_id.is_some() {
                        return Err(YubiKeyError::InvalidResponse(
                            "SELECT response contains duplicate device IDs".into(),
                        ));
                    }
                    let parsed_device_id = <[u8; 8]>::try_from(tlv.value).map_err(|_| {
                        YubiKeyError::InvalidResponse(
                            "SELECT response contains an invalid device ID length".into(),
                        )
                    })?;
                    device_id = Some(parsed_device_id);
                }
                Some(Tag::Name) => {
                    return Err(YubiKeyError::InvalidResponse(
                        "SELECT response contains an invalid device ID length".into(),
                    ));
                }
                Some(Tag::Challenge) => {
                    if challenge.is_some() || tlv.value.len() != 8 {
                        return Err(YubiKeyError::InvalidResponse(
                            "Invalid authentication challenge".into(),
                        ));
                    }
                    challenge = Some(tlv.value.to_vec());
                }
                Some(Tag::Algorithm) => {
                    if algo.is_some() || tlv.value.len() != 1 {
                        return Err(YubiKeyError::InvalidResponse(
                            "Invalid authentication algorithm".into(),
                        ));
                    }
                    let parsed = apdu::Algorithm::from_byte(tlv.value[0])
                        .filter(|a| *a as u8 == tlv.value[0]);
                    algo = Some(parsed.ok_or_else(|| {
                        YubiKeyError::InvalidResponse("Unsupported authentication algorithm".into())
                    })?);
                }
                _ => {}
            }
        }

        if !parser.is_empty() {
            return Err(YubiKeyError::InvalidResponse(
                "SELECT response contains malformed TLV data".into(),
            ));
        }

        let device_id = device_id.ok_or_else(|| {
            YubiKeyError::InvalidResponse("SELECT response is missing the device ID".into())
        })?;

        Ok((version, device_id, challenge, algo))
    }

    /// Check if the YubiKey requires authentication
    pub fn requires_auth(&self) -> bool {
        self.challenge.is_some()
    }

    /// Transmit an APDU and receive a response
    pub fn transmit(&self, apdu: &[u8]) -> Result<Vec<u8>> {
        let mut recv_buf = [0u8; MAX_BUFFER_SIZE];
        let mut card = self.card.borrow_mut();
        let transaction = card.transaction()?;
        let response = transaction.transmit(apdu, &mut recv_buf)?;
        let mut frames = 0;

        if response.len() < 2 {
            return Err(YubiKeyError::InvalidResponse("Response too short".into()));
        }

        let mut full_response = response.to_vec();

        // Handle "more data available" responses
        while full_response.len() >= 2 {
            let sw1 = full_response[full_response.len() - 2];
            if sw1 == sw::MORE_DATA {
                frames += 1;
                if frames > 64 || full_response.len() > 65536 {
                    return Err(YubiKeyError::InvalidResponse(
                        "Response exceeds safety limit".into(),
                    ));
                }
                // Remove status bytes from accumulated response
                full_response.truncate(full_response.len() - 2);

                // Send GET REMAINING command
                let remaining_apdu = apdu::build_send_remaining_apdu();
                let more_response = transaction.transmit(&remaining_apdu, &mut recv_buf)?;
                if more_response.len() < 2 {
                    return Err(YubiKeyError::InvalidResponse(
                        "Truncated continuation frame".into(),
                    ));
                }
                full_response.extend_from_slice(more_response);
            } else {
                break;
            }
        }

        Ok(full_response)
    }

    /// Check the status word and return an appropriate error if not success
    pub fn check_status<'a>(&self, response: &'a [u8]) -> Result<&'a [u8]> {
        if response.len() < 2 {
            return Err(YubiKeyError::InvalidResponse("Response too short".into()));
        }

        let sw1 = response[response.len() - 2];
        let sw2 = response[response.len() - 1];

        match (sw1, sw2) {
            sw::SUCCESS => Ok(&response[..response.len() - 2]),
            sw::AUTH_REQUIRED => Err(YubiKeyError::AuthenticationRequired),
            sw::NO_SUCH_OBJECT => Err(YubiKeyError::CredentialNotFound("Unknown".into())),
            sw::NO_SPACE => Err(YubiKeyError::NoSpace),
            sw::WRONG_SYNTAX => Err(YubiKeyError::InvalidName),
            sw::AUTH_NOT_INITIALIZED => Err(YubiKeyError::WrongPassword),
            _ => Err(YubiKeyError::ApduError { sw1, sw2 }),
        }
    }

    /// Get the serial number of the connected YubiKey (if available via ATR)
    #[allow(dead_code)]
    pub fn get_atr(&self) -> Result<Vec<u8>> {
        let mut atr_buf = [0u8; MAX_ATR_SIZE];
        let mut reader_names_buf = [0u8; 256];

        let status = self
            .card
            .borrow()
            .status2(&mut reader_names_buf, &mut atr_buf)?;

        Ok(status.atr().to_vec())
    }
}

/// Find all available YubiKey reader names
#[allow(dead_code)]
pub fn list_yubikey_readers() -> Result<Vec<String>> {
    let ctx = Context::establish(Scope::User)?;
    let mut readers_buf = [0u8; 2048];
    let readers = ctx.list_readers(&mut readers_buf)?;

    let reader_names: Vec<String> = readers.map(|r| r.to_string_lossy().into_owned()).collect();

    Ok(reader_names)
}

/// Owned by a single device worker. Tests inject deterministic APDU responses.
pub trait OathConnection: Send {
    fn is_present(&self) -> Result<bool>;
    fn version(&self) -> (u8, u8, u8);
    fn device_id(&self) -> [u8; 8];
    fn challenge(&self) -> Option<&[u8]>;
    fn challenge_algorithm(&self) -> apdu::Algorithm {
        apdu::Algorithm::Sha1
    }
    fn transmit(&self, command: &[u8]) -> Result<Vec<u8>>;
}
impl OathConnection for YubiKeyConnection {
    fn is_present(&self) -> Result<bool> {
        let status = self.card.borrow().status2_owned()?;
        Ok(status.status().intersects(
            pcsc::Status::PRESENT
                | pcsc::Status::SWALLOWED
                | pcsc::Status::POWERED
                | pcsc::Status::NEGOTIABLE
                | pcsc::Status::SPECIFIC,
        ))
    }
    fn version(&self) -> (u8, u8, u8) {
        self.version
    }
    fn device_id(&self) -> [u8; 8] {
        self.device_id
    }
    fn challenge(&self) -> Option<&[u8]> {
        self.challenge.as_deref()
    }
    fn challenge_algorithm(&self) -> apdu::Algorithm {
        self.challenge_algorithm.unwrap_or(apdu::Algorithm::Sha1)
    }
    fn transmit(&self, command: &[u8]) -> Result<Vec<u8>> {
        YubiKeyConnection::transmit(self, command)
    }
}
pub fn response_data(response: &[u8]) -> Result<&[u8]> {
    if response.len() < 2 {
        return Err(YubiKeyError::InvalidResponse("Response too short".into()));
    }
    let status = (response[response.len() - 2], response[response.len() - 1]);
    match status {
        sw::SUCCESS => Ok(&response[..response.len() - 2]),
        sw::AUTH_REQUIRED => Err(YubiKeyError::AuthenticationRequired),
        sw::NO_SUCH_OBJECT => Err(YubiKeyError::CredentialNotFound(
            "Credential is not available on this key".into(),
        )),
        sw::NO_SPACE => Err(YubiKeyError::NoSpace),
        sw::WRONG_SYNTAX => Err(YubiKeyError::InvalidName),
        sw::AUTH_NOT_INITIALIZED => Err(YubiKeyError::WrongPassword),
        (sw1, sw2) => Err(YubiKeyError::ApduError { sw1, sw2 }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn successful_select_response(data: &[u8]) -> Vec<u8> {
        let mut response = data.to_vec();
        response.extend_from_slice(&[sw::SUCCESS.0, sw::SUCCESS.1]);
        response
    }

    #[test]
    fn select_rejects_invalid_or_duplicate_authentication_metadata() {
        for extra in [
            vec![0x7b, 1, 0x21],
            vec![0x7b, 0],
            vec![0x74, 1, 7],
            vec![0x7b, 1, 1, 0x7b, 1, 2],
        ] {
            let mut data = vec![0x79, 3, 5, 7, 1, 0x71, 8];
            data.extend_from_slice(b"12345678");
            data.extend(extra);
            assert!(
                YubiKeyConnection::parse_select_response(&successful_select_response(&data))
                    .is_err()
            );
        }
    }
    #[test]
    fn select_response_requires_device_id() {
        let response = successful_select_response(&[Tag::Version as u8, 3, 5, 7, 1]);

        assert!(matches!(
            YubiKeyConnection::parse_select_response(&response),
            Err(YubiKeyError::InvalidResponse(message)) if message.contains("missing the device ID")
        ));
    }

    #[test]
    fn select_response_rejects_invalid_device_id_length() {
        let response = successful_select_response(&[Tag::Name as u8, 7, 1, 2, 3, 4, 5, 6, 7]);

        assert!(matches!(
            YubiKeyConnection::parse_select_response(&response),
            Err(YubiKeyError::InvalidResponse(message)) if message.contains("invalid device ID length")
        ));
    }

    #[test]
    fn select_response_rejects_malformed_tlv() {
        let response = successful_select_response(&[Tag::Name as u8, 8, 1, 2, 3]);

        assert!(matches!(
            YubiKeyConnection::parse_select_response(&response),
            Err(YubiKeyError::InvalidResponse(message)) if message.contains("malformed TLV")
        ));
    }

    #[test]
    fn select_response_accepts_exact_device_id() {
        let response = successful_select_response(&[
            Tag::Version as u8,
            3,
            5,
            7,
            1,
            Tag::Name as u8,
            8,
            1,
            2,
            3,
            4,
            5,
            6,
            7,
            8,
        ]);

        let (version, device_id, challenge, algorithm) =
            YubiKeyConnection::parse_select_response(&response).unwrap();
        assert_eq!(version, (5, 7, 1));
        assert_eq!(device_id, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(challenge.is_none());
        assert!(algorithm.is_none());
    }

    #[test]
    #[ignore] // Requires actual YubiKey
    fn test_connection() {
        let conn = YubiKeyConnection::connect().unwrap();
        println!("Version: {:?}", conn.version);
        println!("Device ID: {:02X?}", conn.device_id);
        println!("Requires auth: {}", conn.requires_auth());
    }
}
