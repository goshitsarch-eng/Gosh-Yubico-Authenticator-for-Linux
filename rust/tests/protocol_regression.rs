//! Protocol regressions use an injected transport, never a production fake device.
use gosh_authenticator_core::core::{
    credential::{CredentialId, NewCredential},
    yubikey::{
        apdu,
        connection::OathConnection,
        error::Result,
        oath::{Calculation, OathSession},
        Algorithm, OathType,
    },
};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::{collections::VecDeque, sync::Mutex};

type Reply = Box<dyn FnOnce(&[u8]) -> Vec<u8> + Send>;
struct ScriptedCard {
    replies: Mutex<VecDeque<Reply>>,
    protected: bool,
}
impl OathConnection for ScriptedCard {
    fn is_present(&self) -> gosh_authenticator_core::core::yubikey::error::Result<bool> {
        Ok(true)
    }
    fn version(&self) -> (u8, u8, u8) {
        (5, 7, 1)
    }
    fn device_id(&self) -> [u8; 8] {
        *b"12345678"
    }
    fn challenge(&self) -> Option<&[u8]> {
        self.protected.then_some(b"abcdefgh".as_slice())
    }
    fn transmit(&self, command: &[u8]) -> Result<Vec<u8>> {
        let reply = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("Unexpected APDU");
        Ok(reply(command))
    }
}
fn success(mut data: Vec<u8>) -> Vec<u8> {
    data.extend_from_slice(&[0x90, 0]);
    data
}
fn session(replies: Vec<Reply>, protected: bool) -> OathSession {
    OathSession::new(ScriptedCard {
        replies: Mutex::new(replies.into()),
        protected,
    })
}

#[test]
fn mutual_authentication_uses_yubico_key_and_rejects_bad_proof() {
    let reply: Reply = Box::new(|command| {
        assert_eq!(command[1], 0xa3);
        let mut key = [0; 16];
        pbkdf2::pbkdf2_hmac::<Sha1>(b"password", b"12345678", 1000, &mut key);
        let fields: Vec<_> = apdu::TlvParser::new(&command[5..]).collect();
        let mut mac = Hmac::<Sha1>::new_from_slice(&key).unwrap();
        mac.update(b"abcdefgh");
        mac.verify_slice(fields[0].value).unwrap();
        let mut proof = Hmac::<Sha1>::new_from_slice(&key).unwrap();
        proof.update(fields[1].value);
        let mut data = vec![0x75, 20];
        data.extend_from_slice(&proof.finalize().into_bytes());
        success(data)
    });
    let mut card = session(vec![reply], true);
    assert!(card.requires_auth());
    card.validate("password").unwrap();
    assert!(!card.requires_auth());
    let mut bad = session(
        vec![Box::new(|_| {
            success(vec![
                0x75, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ])
        })],
        true,
    );
    assert!(bad.validate("password").is_err());
    assert!(bad.requires_auth());
}

#[test]
fn touch_encoding_counter_and_nondefault_period_are_preserved() {
    let list: Reply = Box::new(|command| {
        assert_eq!(command[1], 0xa1);
        success(vec![])
    });
    let put: Reply = Box::new(|command| {
        assert_eq!(command[1], 1);
        assert_eq!(command[4] as usize, command.len() - 5);
        assert!(command.windows(2).any(|b| b == [0x78, 0x02]));
        assert!(!command.windows(3).any(|b| b == [0x78, 0x01, 0x02]));
        assert!(command.ends_with(&[0x7a, 4, 0, 0, 0, 42]));
        success(vec![])
    });
    let card = session(vec![list, put], false);
    let mut new = NewCredential::default();
    new.account = "alice".into();
    new.secret = b"12345678901234567890".to_vec();
    new.oath_type = OathType::Hotp;
    new.initial_counter = Some(42);
    new.require_touch = true;

    let credential = card.put(&new).unwrap();
    assert!(credential.is_hotp());
    let mut totp = NewCredential::default();
    totp.issuer = Some("服务".into());
    totp.account = "é".into();
    totp.secret = vec![1; 20];
    totp.period = 60;

    assert_eq!(totp.name(), "60/服务:é".as_bytes());
}

#[test]
fn password_command_uses_totp_type_and_tracks_removal() {
    let set: Reply = Box::new(|command| {
        assert_eq!(&command[5..8], &[0x73, 17, 0x21]);
        success(vec![])
    });
    let unset: Reply = Box::new(|command| {
        assert_eq!(&command[5..], &[0x73, 0]);
        success(vec![])
    });
    let mut card = session(vec![set, unset], false);
    card.set_code("long password").unwrap();
    assert!(card.has_password());
    card.set_code("").unwrap();
    assert!(!card.has_password());
}

#[test]
fn calculate_all_distinguishes_hotp_from_touch_and_rejects_truncation() {
    let card = session(
        vec![Box::new(|_| {
            success(vec![0x71, 1, b'a', 0x77, 0, 0x71, 1, b'b', 0x7c, 0])
        })],
        false,
    );
    assert_eq!(
        card.calculate_all(Some(120)).unwrap(),
        vec![
            (CredentialId(vec![b'a']), Calculation::Hotp),
            (CredentialId(vec![b'b']), Calculation::Touch)
        ]
    );
    let bad = session(vec![Box::new(|_| success(vec![0x71, 4, b'a']))], false);
    assert!(bad.calculate_all(Some(120)).is_err());
}

#[test]
fn duplicate_credential_never_overwrites_secret() {
    let card = session(
        vec![Box::new(|_| {
            success(vec![0x72, 6, 0x21, b'a', b'l', b'i', b'c', b'e'])
        })],
        false,
    );
    let mut new = NewCredential::default();
    new.account = "alice".into();
    new.secret = vec![1; 20];
    new.algorithm = Algorithm::Sha1;

    assert!(card.put(&new).is_err());
}

#[test]
fn secrets_are_redacted_from_debug() {
    let mut new = NewCredential::default();
    new.secret = vec![42; 20];
    new.account = "private account".into();

    let text = format!("{new:?}");
    assert!(!text.contains("private account"));
    assert!(!text.contains("42"));
}
