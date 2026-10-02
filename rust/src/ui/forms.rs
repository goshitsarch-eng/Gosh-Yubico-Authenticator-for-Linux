use super::{Page, Ui};
use dioxus::prelude::*;
use gosh_authenticator_core::{
    app::{Command, Connection},
    core::{
        credential::{decode_secret, NewCredential},
        yubikey::{Algorithm, OathType},
    },
    qr::parse_otpauth_uri,
};
#[derive(Clone)]
pub struct AddForm {
    pub issuer: String,
    pub account: String,
    pub secret: String,
    pub kind: String,
    pub algorithm: String,
    pub digits: String,
    pub period: String,
    pub counter: String,
    pub touch: bool,
}
impl Default for AddForm {
    fn default() -> Self {
        Self {
            issuer: String::new(),
            account: String::new(),
            secret: String::new(),
            kind: "totp".into(),
            algorithm: "SHA1".into(),
            digits: "6".into(),
            period: "30".into(),
            counter: "0".into(),
            touch: false,
        }
    }
}
impl Drop for AddForm {
    fn drop(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.secret);
    }
}
impl AddForm {
    pub fn from_credential(c: &NewCredential) -> Self {
        Self {
            issuer: c.issuer.clone().unwrap_or_default(),
            account: c.account.clone(),
            secret: data_encoding::BASE32_NOPAD.encode(&c.secret),
            kind: if c.oath_type == OathType::Hotp {
                "hotp"
            } else {
                "totp"
            }
            .into(),
            algorithm: format!("{:?}", c.algorithm).to_uppercase(),
            digits: c.digits.to_string(),
            period: c.period.to_string(),
            counter: c.initial_counter.unwrap_or(0).to_string(),
            touch: c.require_touch,
        }
    }
    pub fn credential(&self) -> Result<NewCredential, String> {
        let c = NewCredential {
            issuer: (!self.issuer.trim().is_empty()).then(|| self.issuer.trim().to_string()),
            account: self.account.trim().to_string(),
            secret: decode_secret(&self.secret)?,
            oath_type: if self.kind == "hotp" {
                OathType::Hotp
            } else {
                OathType::Totp
            },
            algorithm: match self.algorithm.as_str() {
                "SHA256" => Algorithm::Sha256,
                "SHA512" => Algorithm::Sha512,
                _ => Algorithm::Sha1,
            },
            digits: self.digits.parse().map_err(|_| "Invalid digits")?,
            period: self.period.parse().map_err(|_| "Invalid TOTP period")?,
            initial_counter: if self.kind == "hotp" {
                Some(
                    self.counter
                        .parse()
                        .map_err(|_| "HOTP counter must be a nonnegative 32-bit number")?,
                )
            } else {
                None
            },
            require_touch: self.touch,
        };
        c.validate()?;
        Ok(c)
    }
}
#[component]
fn TextField(
    label: &'static str,
    value: String,
    password: bool,
    oninput: EventHandler<String>,
) -> Element {
    let kind = if password { "password" } else { "text" };
    rsx! {
        label { class: "field",
            span { "{label}" }
            input {
                r#type: kind,
                maxlength: if password { 8192 } else { 64 },
                value,
                autocomplete: "off",
                oninput: move |e| oninput.call(e.value()),
            }
        }
    }
}
#[component]
pub fn AddCredential() -> Element {
    let ui = use_context::<Ui>();
    let mut form = ui.form;
    let mut page = ui.page;
    let f = form.read().clone();
    let busy = ui.state.read().busy;
    let ready = ui.state.read().connection == Connection::Ready;
    let error = f.credential().err();
    let secret_ui = ui;
    let save_ui = ui;
    rsx! {
        section {
            class: "card form",
            onmounted: move |_| {
                let _ = dioxus_desktop::window().webview.focus();
                spawn(async {
                    let _ = document::eval("document.querySelector('.form input')?.focus()")
                        .await;
                });
            },
            div { class: "form-heading",
                h2 { "Account details" }
                button { onclick: move |_| ui.action("import"), "Import QR image…" }
            }
            p { class: "hint",
                "You can also paste an otpauth:// URI into Secret or drop a QR image onto the window."
            }
            TextField {
                label: "Issuer",
                value: f.issuer.clone(),
                password: false,
                oninput: move |s| form.write().issuer = s,
            }
            TextField {
                label: "Account",
                value: f.account.clone(),
                password: false,
                oninput: move |s| form.write().account = s,
            }
            TextField {
                label: "Secret (Base32 or otpauth URI)",
                value: f.secret.clone(),
                password: true,
                oninput: move |s: String| {
                    if s.trim().starts_with("otpauth://") {
                        match parse_otpauth_uri(&s) {
                            Ok(c) => form.set(AddForm::from_credential(&c)),
                            Err(e) => {
                                form.write().secret = s;
                                secret_ui.error(e.to_string());
                            }
                        }
                    } else {
                        form.write().secret = s;
                    }
                },
            }
            div { class: "form-grid",
                label { class: "field",
                    span { "Type" }
                    select {
                        "aria-label": "Type",
                        value: f.kind.clone(),
                        onchange: move |e| form.write().kind = e.value(),
                        option { value: "totp", selected: f.kind == "totp", "TOTP (time based)" }
                        option { value: "hotp", selected: f.kind == "hotp", "HOTP (counter based)" }
                    }
                }
                label { class: "field",
                    span { "Algorithm" }
                    select {
                        "aria-label": "Algorithm",
                        value: f.algorithm.clone(),
                        onchange: move |e| form.write().algorithm = e.value(),
                        for algorithm in ["SHA1", "SHA256", "SHA512"] {
                            option { value: algorithm, selected: f.algorithm == algorithm, "{algorithm}" }
                        }
                    }
                }
                label { class: "field",
                    span { "Digits" }
                    select {
                        "aria-label": "Digits",
                        value: f.digits.clone(),
                        onchange: move |e| form.write().digits = e.value(),
                        for digits in ["6", "7", "8"] {
                            option { value: digits, selected: f.digits == digits, "{digits}" }
                        }
                    }
                }
                if f.kind == "hotp" {
                    label { class: "field",
                        span { "Initial HOTP counter" }
                        input {
                            r#type: "number",
                            min: "0",
                            max: "4294967295",
                            value: f.counter.clone(),
                            oninput: move |e| form.write().counter = e.value(),
                        }
                    }
                } else {
                    label { class: "field",
                        span { "TOTP period (seconds)" }
                        input {
                            r#type: "number",
                            min: "1",
                            max: "86400",
                            value: f.period.clone(),
                            oninput: move |e| form.write().period = e.value(),
                        }
                    }
                }
            }
            label { class: "check-row",
                input {
                    r#type: "checkbox",
                    checked: f.touch,
                    onchange: move |e| form.write().touch = e.checked(),
                }
                "Require physical touch to generate a code"
            }
            if !ready {
                p { class: "hint",
                    "Connect and unlock a YubiKey before saving. You can prepare or import the form now."
                }
            }
            if let Some(error) = error.clone() {
                p { class: "validation", role: "status", "{error}" }
            }
            div { class: "form-actions",
                button {
                    onclick: move |_| {
                        if ui.state.read().busy {
                            ui.runtime.read().cancel();
                        }
                        page.set(Page::Credentials);
                    },
                    "Cancel"
                }
                button {
                    class: "primary",
                    disabled: !ready || busy || error.is_some(),
                    onclick: move |_| match form.read().credential() {
                        Ok(c) => save_ui.send(Command::Add(c)),
                        Err(e) => save_ui.error(e),
                    },
                    if busy {
                        "Saving…"
                    } else {
                        "Save credential"
                    }
                }
            }
            p { class: "hint",
                "Credentials are stored on your hardware key. This application does not save your secret to disk."
            }
        }
    }
}
