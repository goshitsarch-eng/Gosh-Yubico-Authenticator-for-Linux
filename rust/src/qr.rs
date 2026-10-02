use crate::core::{
    credential::{decode_secret, NewCredential},
    yubikey::{Algorithm, OathType},
};
use std::{collections::HashMap, path::Path};
use thiserror::Error;
#[derive(Debug, Error)]
#[error("{0}")]
pub struct ImportError(pub String);
pub fn parse_otpauth_uri(input: &str) -> Result<NewCredential, ImportError> {
    let fail = |s: &str| ImportError(s.into());
    if input.len() > 8192 {
        return Err(fail("OTP URI is too long"));
    }
    let uri = url::Url::parse(input.trim()).map_err(|_| fail("Invalid OTP URI"))?;
    if uri.scheme() != "otpauth"
        || !uri.username().is_empty()
        || uri.password().is_some()
        || uri.port().is_some()
        || uri.fragment().is_some()
    {
        return Err(fail("Expected an otpauth:// URI"));
    }
    let oath_type = match uri.host_str() {
        Some("totp") => OathType::Totp,
        Some("hotp") => OathType::Hotp,
        _ => return Err(fail("OTP type must be totp or hotp")),
    };
    let label = percent_encoding::percent_decode_str(uri.path().trim_start_matches('/'))
        .decode_utf8()
        .map_err(|_| fail("Account label is not valid UTF-8"))?;
    let (label_issuer, account) = match label.split_once(':') {
        Some((i, a)) => (Some(i.trim().to_string()), a.trim().to_string()),
        None => (None, label.trim().to_string()),
    };
    let mut values = HashMap::new();
    for (k, v) in uri.query_pairs() {
        if values.insert(k.into_owned(), v.into_owned()).is_some() {
            return Err(fail("Duplicate OTP URI parameter"));
        }
    }
    let issuer = values
        .get("issuer")
        .filter(|s| !s.is_empty())
        .cloned()
        .or_else(|| label_issuer.clone());
    if label_issuer
        .as_ref()
        .zip(issuer.as_ref())
        .is_some_and(|(a, b)| a != b)
    {
        return Err(fail("Issuer in label and query must match"));
    }
    let secret = decode_secret(
        values
            .get("secret")
            .ok_or_else(|| fail("Secret key is required"))?,
    )
    .map_err(ImportError)?;
    let algorithm = match values
        .get("algorithm")
        .map(|s| s.to_ascii_uppercase())
        .as_deref()
    {
        None | Some("SHA1") => Algorithm::Sha1,
        Some("SHA256") => Algorithm::Sha256,
        Some("SHA512") => Algorithm::Sha512,
        _ => return Err(fail("Unsupported hash algorithm")),
    };
    let number = |key: &str, default: u32| -> Result<u32, ImportError> {
        values.get(key).map_or(Ok(default), |s| {
            s.parse()
                .map_err(|_| fail("Invalid numeric OTP URI parameter"))
        })
    };
    let digits =
        u8::try_from(number("digits", 6)?).map_err(|_| fail("Invalid number of digits"))?;
    let period = number("period", 30)?;
    let counter = number("counter", 0)?;
    let initial_counter = if oath_type == OathType::Hotp {
        Some(counter)
    } else {
        None
    };
    let new = NewCredential {
        issuer,
        account,
        secret,
        oath_type,
        algorithm,
        digits,
        require_touch: false,
        initial_counter,
        period,
    };
    new.validate().map_err(ImportError)?;
    Ok(new)
}
pub fn scan_qr_file(path: &Path) -> Result<NewCredential, ImportError> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|e| ImportError(e.to_string()))?;
    let metadata = file.metadata().map_err(|e| ImportError(e.to_string()))?;
    if !metadata.is_file() {
        return Err(ImportError("Select a regular image file".into()));
    }
    if metadata.len() > 16 * 1024 * 1024 {
        return Err(ImportError("Image exceeds the 16 MiB limit".into()));
    }
    let mut bytes = Vec::new();
    file.take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| ImportError(e.to_string()))?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(ImportError("Image exceeds the 16 MiB limit".into()));
    }
    let make_reader = || {
        image::ImageReader::new(std::io::Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(|e| ImportError(e.to_string()))
    };
    let (width, height) = make_reader()?
        .into_dimensions()
        .map_err(|_| ImportError("Could not read image dimensions".into()))?;
    if u64::from(width) * u64::from(height) > 16 * 1024 * 1024 {
        return Err(ImportError("Image exceeds the 16 megapixel limit".into()));
    }
    let mut reader = make_reader()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let img = reader
        .decode()
        .map_err(|_| ImportError("Could not decode this image within the size limit".into()))?;
    let mut prepared = rqrr::PreparedImage::prepare(img.to_luma8());
    for grid in prepared.detect_grids() {
        if let Ok((_, content)) = grid.decode() {
            if content.starts_with("otpauth://") {
                return parse_otpauth_uri(&content);
            }
        }
    }
    Err(ImportError(
        "No readable OTP QR code found in this image".into(),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_period_and_counter_preserved() {
        let n = parse_otpauth_uri(
            "otpauth://totp/%E6%9C%8D%E5%8A%A1:%C3%A9?secret=JBSWY3DPEHPK3PXP&period=60",
        )
        .unwrap();
        assert_eq!(n.account, "é");
        assert_eq!(n.period, 60);
        assert_eq!(n.issuer.as_deref(), Some("服务"));
        let n =
            parse_otpauth_uri("otpauth://hotp/alice?secret=JBSWY3DPEHPK3PXP&counter=42").unwrap();
        assert_eq!(n.initial_counter, Some(42));
    }
    #[test]
    fn invalid_parameters_rejected() {
        for tail in [
            "period=0",
            "digits=9",
            "counter=x",
            "secret=ABC&secret=DEF",
            "issuer=Other",
        ] {
            let uri = format!("otpauth://totp/Example:alice?secret=JBSWY3DPEHPK3PXP&{tail}");
            assert!(parse_otpauth_uri(&uri).is_err(), "{tail}");
        }
    }
}
