/// APDU (Application Protocol Data Unit) building and parsing for YubiKey OATH communication
///
/// Based on YKOATH Protocol: https://developers.yubico.com/OATH/YKOATH_Protocol.html

/// OATH applet AID (Application Identifier)
pub const OATH_AID: &[u8] = &[0xA0, 0x00, 0x00, 0x05, 0x27, 0x21, 0x01];

/// OATH instruction codes
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Instruction {
    Put = 0x01,
    Delete = 0x02,
    SetCode = 0x03,
    Reset = 0x04,
    List = 0xA1,
    Calculate = 0xA2,
    Validate = 0xA3,
    CalculateAll = 0xA4,
    SendRemaining = 0xA5,
}

/// TLV (Tag-Length-Value) tags used in OATH protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Name = 0x71,
    NameList = 0x72,
    Key = 0x73,
    Challenge = 0x74,
    Response = 0x75,
    TruncatedResponse = 0x76,
    Hotp = 0x77,
    Property = 0x78,
    Version = 0x79,
    InitialMovingFactor = 0x7A,
    Algorithm = 0x7B,
    Touch = 0x7C,
}

impl Tag {
    pub fn from_byte(byte: u8) -> Option<Tag> {
        match byte {
            0x71 => Some(Tag::Name),
            0x72 => Some(Tag::NameList),
            0x73 => Some(Tag::Key),
            0x74 => Some(Tag::Challenge),
            0x75 => Some(Tag::Response),
            0x76 => Some(Tag::TruncatedResponse),
            0x77 => Some(Tag::Hotp),
            0x78 => Some(Tag::Property),
            0x79 => Some(Tag::Version),
            0x7A => Some(Tag::InitialMovingFactor),
            0x7B => Some(Tag::Algorithm),
            0x7C => Some(Tag::Touch),
            _ => None,
        }
    }
}

/// OATH algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Algorithm {
    #[default]
    Sha1 = 0x01,
    Sha256 = 0x02,
    Sha512 = 0x03,
}

impl Algorithm {
    pub fn from_byte(byte: u8) -> Option<Algorithm> {
        match byte & 0x0F {
            0x01 => Some(Algorithm::Sha1),
            0x02 => Some(Algorithm::Sha256),
            0x03 => Some(Algorithm::Sha512),
            _ => None,
        }
    }

    pub fn hmac_key_size(&self) -> usize {
        match self {
            Algorithm::Sha1 => 20,
            Algorithm::Sha256 => 32,
            Algorithm::Sha512 => 64,
        }
    }
}

/// OATH type (TOTP or HOTP)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OathType {
    Hotp = 0x10,
    #[default]
    Totp = 0x20,
}

impl OathType {
    pub fn from_byte(byte: u8) -> Option<OathType> {
        match byte & 0xF0 {
            0x10 => Some(OathType::Hotp),
            0x20 => Some(OathType::Totp),
            _ => None,
        }
    }
}

/// Build a SELECT APDU for the OATH applet
pub fn build_select_apdu() -> Vec<u8> {
    let mut apdu = vec![
        0x00, // CLA
        0xA4, // INS: SELECT
        0x04, // P1: Select by DF name
        0x00, // P2
        OATH_AID.len() as u8,
    ];
    apdu.extend_from_slice(OATH_AID);
    apdu
}

/// Build a LIST credentials APDU
pub fn build_list_apdu() -> Vec<u8> {
    vec![
        0x00,                     // CLA
        Instruction::List as u8, // INS
        0x00,                     // P1
        0x00,                     // P2
    ]
}

/// Build a CALCULATE APDU for a single credential
pub fn build_calculate_apdu(name: &[u8], challenge: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();

    // Name TLV
    data.push(Tag::Name as u8);
    data.push(name.len() as u8);
    data.extend_from_slice(name);

    // Challenge TLV
    data.push(Tag::Challenge as u8);
    data.push(challenge.len() as u8);
    data.extend_from_slice(challenge);

    let mut apdu = vec![
        0x00,                          // CLA
        Instruction::Calculate as u8, // INS
        0x00,                          // P1
        0x01,                          // P2: truncate
        data.len() as u8,
    ];
    apdu.extend(data);
    apdu
}

/// Build a CALCULATE ALL APDU (calculates all TOTP credentials at once)
pub fn build_calculate_all_apdu(challenge: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();

    // Challenge TLV
    data.push(Tag::Challenge as u8);
    data.push(challenge.len() as u8);
    data.extend_from_slice(challenge);

    let mut apdu = vec![
        0x00,                             // CLA
        Instruction::CalculateAll as u8, // INS
        0x00,                             // P1
        0x01,                             // P2: truncate
        data.len() as u8,
    ];
    apdu.extend(data);
    apdu
}

/// Build a PUT (add credential) APDU
pub fn build_put_apdu(
    name: &[u8],
    secret: &[u8],
    oath_type: OathType,
    algorithm: Algorithm,
    digits: u8,
    require_touch: bool,
    initial_counter: Option<u32>,
) -> Vec<u8> {
    let mut data = Vec::new();

    // Name TLV
    data.push(Tag::Name as u8);
    data.push(name.len() as u8);
    data.extend_from_slice(name);

    // Key TLV: type_algo byte + digits byte + secret
    let type_algo = (oath_type as u8) | (algorithm as u8);
    let key_data_len = 2 + secret.len();
    data.push(Tag::Key as u8);
    data.push(key_data_len as u8);
    data.push(type_algo);
    data.push(digits);
    data.extend_from_slice(secret);

    // Property TLV (if touch required)
    if require_touch {
        data.push(Tag::Property as u8);
        data.push(0x01); // Length = 1 byte
        data.push(0x02); // REQUIRE_TOUCH = 0x02
    }

    // Initial moving factor (for HOTP)
    if let Some(counter) = initial_counter {
        data.push(Tag::InitialMovingFactor as u8);
        data.push(4);
        data.extend_from_slice(&counter.to_be_bytes());
    }

    let mut apdu = vec![
        0x00,                    // CLA
        Instruction::Put as u8, // INS
        0x00,                    // P1
        0x00,                    // P2
        data.len() as u8,
    ];
    apdu.extend(data);
    apdu
}

/// Build a DELETE credential APDU
pub fn build_delete_apdu(name: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();

    // Name TLV
    data.push(Tag::Name as u8);
    data.push(name.len() as u8);
    data.extend_from_slice(name);

    let mut apdu = vec![
        0x00,                       // CLA
        Instruction::Delete as u8, // INS
        0x00,                       // P1
        0x00,                       // P2
        data.len() as u8,
    ];
    apdu.extend(data);
    apdu
}

/// Build a VALIDATE (authenticate) APDU
pub fn build_validate_apdu(response: &[u8], challenge: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();

    // Response TLV
    data.push(Tag::Response as u8);
    data.push(response.len() as u8);
    data.extend_from_slice(response);

    // Challenge TLV
    data.push(Tag::Challenge as u8);
    data.push(challenge.len() as u8);
    data.extend_from_slice(challenge);

    let mut apdu = vec![
        0x00,                         // CLA
        Instruction::Validate as u8, // INS
        0x00,                         // P1
        0x00,                         // P2
        data.len() as u8,
    ];
    apdu.extend(data);
    apdu
}

/// Build a SEND REMAINING APDU (for getting more response data)
pub fn build_send_remaining_apdu() -> Vec<u8> {
    vec![
        0x00,                              // CLA
        Instruction::SendRemaining as u8, // INS
        0x00,                              // P1
        0x00,                              // P2
    ]
}

/// Build a SET CODE (change password) APDU
///
/// If key is empty, this removes the password protection.
/// Otherwise, sets a new password using the derived key.
pub fn build_set_code_apdu(key: &[u8], challenge: &[u8], response: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();

    // Key TLV (algorithm + key)
    // For removing password: key is empty
    data.push(Tag::Key as u8);
    if key.is_empty() {
        data.push(0x00); // Length = 0 means remove password
    } else {
        data.push((1 + key.len()) as u8); // algorithm byte + key
        data.push(Algorithm::Sha1 as u8); // Always use SHA1 for password
        data.extend_from_slice(key);
    }

    // Challenge TLV (for verification)
    if !challenge.is_empty() {
        data.push(Tag::Challenge as u8);
        data.push(challenge.len() as u8);
        data.extend_from_slice(challenge);
    }

    // Response TLV (for verification)
    if !response.is_empty() {
        data.push(Tag::Response as u8);
        data.push(response.len() as u8);
        data.extend_from_slice(response);
    }

    let mut apdu = vec![
        0x00,                        // CLA
        Instruction::SetCode as u8, // INS
        0x00,                        // P1
        0x00,                        // P2
        data.len() as u8,
    ];
    apdu.extend(data);
    apdu
}

/// Parse TLV data from a response buffer
pub struct TlvParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> TlvParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[allow(dead_code)]
    pub fn remaining(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }
}

/// A single TLV entry
#[derive(Debug)]
pub struct Tlv<'a> {
    pub tag: u8,
    pub value: &'a [u8],
}

impl<'a> Iterator for TlvParser<'a> {
    type Item = Tlv<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 2 > self.data.len() {
            return None;
        }

        let tag = self.data[self.pos];
        let len = self.data[self.pos + 1] as usize;

        if self.pos + 2 + len > self.data.len() {
            return None;
        }

        let value = &self.data[self.pos + 2..self.pos + 2 + len];
        self.pos += 2 + len;

        Some(Tlv { tag, value })
    }
}

/// Status word constants
pub mod sw {
    pub const SUCCESS: (u8, u8) = (0x90, 0x00);
    pub const MORE_DATA: u8 = 0x61;
    pub const AUTH_REQUIRED: (u8, u8) = (0x69, 0x82);
    pub const WRONG_SYNTAX: (u8, u8) = (0x6A, 0x80);
    pub const NO_SUCH_OBJECT: (u8, u8) = (0x69, 0x84);
    pub const AUTH_NOT_INITIALIZED: (u8, u8) = (0x69, 0x86);
    pub const NO_SPACE: (u8, u8) = (0x6A, 0x84);
}

/// Get the current TOTP challenge (time-based)
#[allow(dead_code)]
pub fn get_totp_challenge(period: u32) -> [u8; 8] {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let counter = timestamp / period as u64;
    counter.to_be_bytes()
}

/// Format an OTP code with spaces for readability
#[allow(dead_code)]
pub fn format_code(code: u32, digits: u8) -> String {
    let code_str = format!("{:0width$}", code, width = digits as usize);
    if digits == 6 {
        format!("{} {}", &code_str[..3], &code_str[3..])
    } else if digits == 8 {
        format!("{} {}", &code_str[..4], &code_str[4..])
    } else {
        code_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_apdu() {
        let apdu = build_select_apdu();
        assert_eq!(apdu[0], 0x00); // CLA
        assert_eq!(apdu[1], 0xA4); // INS
        assert_eq!(apdu[4], 7);    // Length of AID
        assert_eq!(&apdu[5..], OATH_AID);
    }

    #[test]
    fn test_list_apdu() {
        let apdu = build_list_apdu();
        assert_eq!(apdu, vec![0x00, 0xA1, 0x00, 0x00]);
    }

    #[test]
    fn test_tlv_parser() {
        let data = [
            0x71, 0x04, b't', b'e', b's', b't', // Name TLV
            0x79, 0x03, 0x05, 0x03, 0x00,       // Version TLV
        ];
        let parser = TlvParser::new(&data);
        let tlvs: Vec<_> = parser.collect();

        assert_eq!(tlvs.len(), 2);
        assert_eq!(tlvs[0].tag, 0x71);
        assert_eq!(tlvs[0].value, b"test");
        assert_eq!(tlvs[1].tag, 0x79);
        assert_eq!(tlvs[1].value, &[0x05, 0x03, 0x00]);
    }

    #[test]
    fn test_format_code() {
        assert_eq!(format_code(123456, 6), "123 456");
        assert_eq!(format_code(12345678, 8), "1234 5678");
        assert_eq!(format_code(1234567, 7), "1234567");
    }
}
