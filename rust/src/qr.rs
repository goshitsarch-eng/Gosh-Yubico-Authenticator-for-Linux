use gosh_authenticator_core::core::yubikey::{Algorithm, OathType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtpAuthCredential {
    pub oath_type: OathType,
    pub issuer: Option<String>,
    pub account: String,
    pub secret: String,
    pub algorithm: Algorithm,
    pub digits: u8,
    pub period: Option<u32>,
    pub counter: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrScanError(pub String);

impl std::fmt::Display for QrScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for QrScanError {}

/// Decode the first QR code in an image file and parse an otpauth:// URI.
pub fn scan_qr_file(path: &std::path::Path) -> Result<OtpAuthCredential, QrScanError> {
    let img = image::open(path).map_err(|_| QrScanError("Could not decode image".into()))?;
    let luma = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();
    if grids.is_empty() {
        return Err(QrScanError("No QR code found in image".into()));
    }
    let (_meta, content) = grids[0]
        .decode()
        .map_err(|_| QrScanError("Could not read QR code".into()))?;
    if content.is_empty() {
        return Err(QrScanError("QR code is empty".into()));
    }
    parse_otpauth_uri(&content)
}

/// Parse `otpauth://TYPE/LABEL?PARAMETERS`.
pub fn parse_otpauth_uri(uri: &str) -> Result<OtpAuthCredential, QrScanError> {
    let parsed = url::Url::parse(uri).or_else(|_| {
        // `otpauth` is accepted by Url; fall back to a manual parse on failure.
        Err(QrScanError("Invalid URI format".into()))
    })?;

    if parsed.scheme() != "otpauth" {
        return Err(QrScanError(format!(
            "Not an otpauth URI (got {}://)",
            parsed.scheme()
        )));
    }

    let oath_type = match parsed.host_str().unwrap_or_default().to_ascii_lowercase().as_str()
    {
        "totp" => OathType::Totp,
        "hotp" => OathType::Hotp,
        other => return Err(QrScanError(format!("Unknown OTP type: {other}"))),
    };

    let mut path = parsed.path().trim_start_matches('/').to_string();
    path = percent_decode(&path);

    let (mut issuer, account) = if let Some((left, right)) = path.split_once(':') {
        (Some(left.trim().to_string()), right.trim().to_string())
    } else {
        (None, path.trim().to_string())
    };

    if account.is_empty() {
        return Err(QrScanError("Account name is required".into()));
    }

    let params: std::collections::HashMap<String, String> = parsed
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let secret = params
        .get("secret")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| QrScanError("Secret key is required".into()))?;

    if let Some(value) = params.get("issuer").filter(|s| !s.is_empty()) {
        issuer = Some(value.clone());
    }

    let algorithm = match params
        .get("algorithm")
        .map(|s| s.to_ascii_uppercase())
        .as_deref()
    {
        None => Algorithm::Sha1,
        Some("SHA1") => Algorithm::Sha1,
        Some("SHA256") => Algorithm::Sha256,
        Some("SHA512") => Algorithm::Sha512,
        Some(other) => {
            return Err(QrScanError(format!("Unsupported algorithm: {other}")));
        }
    };

    let digits = match params.get("digits") {
        None => 6,
        Some(value) => {
            let parsed = value
                .parse::<u8>()
                .map_err(|_| QrScanError("Digits must be 6, 7, or 8".into()))?;
            if !(6..=8).contains(&parsed) {
                return Err(QrScanError("Digits must be 6, 7, or 8".into()));
            }
            parsed
        }
    };

    let period = if oath_type == OathType::Totp {
        params.get("period").and_then(|v| v.parse().ok())
    } else {
        None
    };

    let counter = if oath_type == OathType::Hotp {
        Some(
            params
                .get("counter")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
        )
    } else {
        None
    };

    Ok(OtpAuthCredential {
        oath_type,
        issuer,
        account,
        secret,
        algorithm,
        digits,
        period,
        counter,
    })
}

fn percent_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push(char::from((h << 4) | l));
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_totp_uri() {
        let cred = parse_otpauth_uri(
            "otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP&issuer=Example&algorithm=SHA256&digits=8&period=30",
        )
        .unwrap();
        assert_eq!(cred.oath_type, OathType::Totp);
        assert_eq!(cred.issuer.as_deref(), Some("Example"));
        assert_eq!(cred.account, "alice@google.com");
        assert_eq!(cred.secret, "JBSWY3DPEHPK3PXP");
        assert_eq!(cred.algorithm, Algorithm::Sha256);
        assert_eq!(cred.digits, 8);
        assert_eq!(cred.period, Some(30));
        assert_eq!(cred.counter, None);
    }

    #[test]
    fn parse_hotp_uri() {
        let cred = parse_otpauth_uri("otpauth://hotp/alice?secret=MFRGGZDF&counter=5").unwrap();
        assert_eq!(cred.oath_type, OathType::Hotp);
        assert_eq!(cred.account, "alice");
        assert_eq!(cred.counter, Some(5));
    }

    #[test]
    fn reject_non_otpauth() {
        let err = parse_otpauth_uri("https://example.com").unwrap_err();
        assert!(err.0.contains("otpauth"));
    }
}
