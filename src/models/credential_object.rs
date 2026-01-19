use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::core::credential::{Credential, CredentialId};
use crate::core::yubikey::{Algorithm, OathType};

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct CredentialObject {
        pub id: RefCell<Vec<u8>>,
        pub issuer: RefCell<Option<String>>,
        pub account: RefCell<String>,
        pub code: RefCell<Option<String>>,
        pub oath_type: Cell<u8>,
        pub algorithm: Cell<u8>,
        pub digits: Cell<u8>,
        pub touch_required: Cell<bool>,
        pub period: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CredentialObject {
        const NAME: &'static str = "GoshCredentialObject";
        type Type = super::CredentialObject;
        type ParentType = Object;
    }

    impl ObjectImpl for CredentialObject {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("issuer").read_only().build(),
                    glib::ParamSpecString::builder("account").read_only().build(),
                    glib::ParamSpecString::builder("code").read_only().build(),
                    glib::ParamSpecBoolean::builder("is-totp").read_only().build(),
                    glib::ParamSpecBoolean::builder("is-hotp").read_only().build(),
                    glib::ParamSpecBoolean::builder("touch-required")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("digits")
                        .minimum(6)
                        .maximum(8)
                        .default_value(6)
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("period")
                        .minimum(15)
                        .maximum(60)
                        .default_value(30)
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "issuer" => obj.issuer().to_value(),
                "account" => obj.account().to_value(),
                "code" => obj.code().to_value(),
                "is-totp" => obj.is_totp().to_value(),
                "is-hotp" => obj.is_hotp().to_value(),
                "touch-required" => obj.touch_required().to_value(),
                "digits" => (obj.digits() as u32).to_value(),
                "period" => obj.period().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct CredentialObject(ObjectSubclass<imp::CredentialObject>);
}

impl CredentialObject {
    pub fn new(credential: &Credential) -> Self {
        let obj: Self = Object::builder().build();
        obj.update_from(credential);
        obj
    }

    pub fn update_from(&self, credential: &Credential) {
        let imp = self.imp();
        imp.id.replace(credential.id.0.clone());
        imp.issuer.replace(credential.issuer.clone());
        imp.account.replace(credential.account.clone());
        imp.code.replace(credential.code.clone());
        imp.oath_type.set(credential.oath_type as u8);
        imp.algorithm.set(credential.algorithm as u8);
        imp.digits.set(credential.digits);
        imp.touch_required.set(credential.touch_required);
        imp.period.set(credential.period);

        // Notify property changes
        self.notify("issuer");
        self.notify("account");
        self.notify("code");
        self.notify("is-totp");
        self.notify("is-hotp");
        self.notify("touch-required");
        self.notify("digits");
        self.notify("period");
    }

    pub fn update_code(&self, code: Option<String>) {
        self.imp().code.replace(code);
        self.notify("code");
    }

    pub fn id(&self) -> CredentialId {
        CredentialId(self.imp().id.borrow().clone())
    }

    pub fn issuer(&self) -> Option<String> {
        self.imp().issuer.borrow().clone()
    }

    pub fn account(&self) -> String {
        self.imp().account.borrow().clone()
    }

    pub fn code(&self) -> Option<String> {
        self.imp().code.borrow().clone()
    }

    pub fn is_totp(&self) -> bool {
        self.imp().oath_type.get() == OathType::Totp as u8
    }

    pub fn is_hotp(&self) -> bool {
        self.imp().oath_type.get() == OathType::Hotp as u8
    }

    pub fn touch_required(&self) -> bool {
        self.imp().touch_required.get()
    }

    pub fn digits(&self) -> u8 {
        self.imp().digits.get()
    }

    pub fn period(&self) -> u32 {
        self.imp().period.get()
    }

    pub fn display_name(&self) -> String {
        match self.issuer() {
            Some(issuer) => format!("{}: {}", issuer, self.account()),
            None => self.account(),
        }
    }

    /// Convert back to a Credential struct
    pub fn to_credential(&self) -> Credential {
        let imp = self.imp();
        Credential {
            id: CredentialId(imp.id.borrow().clone()),
            issuer: imp.issuer.borrow().clone(),
            account: imp.account.borrow().clone(),
            oath_type: if self.is_totp() {
                OathType::Totp
            } else {
                OathType::Hotp
            },
            algorithm: Algorithm::from_byte(imp.algorithm.get()).unwrap_or(Algorithm::Sha1),
            digits: imp.digits.get(),
            touch_required: imp.touch_required.get(),
            code: imp.code.borrow().clone(),
            period: imp.period.get(),
        }
    }
}

impl Default for CredentialObject {
    fn default() -> Self {
        Object::builder().build()
    }
}
