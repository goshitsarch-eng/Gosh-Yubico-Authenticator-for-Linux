use gio::ListModel;
use glib::Object;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::core::credential::{Credential, CredentialId};
use crate::models::CredentialObject;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct CredentialList {
        pub credentials: RefCell<Vec<CredentialObject>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CredentialList {
        const NAME: &'static str = "GoshCredentialList";
        type Type = super::CredentialList;
        type ParentType = Object;
        type Interfaces = (ListModel,);
    }

    impl ObjectImpl for CredentialList {}

    impl ListModelImpl for CredentialList {
        fn item_type(&self) -> glib::Type {
            CredentialObject::static_type()
        }

        fn n_items(&self) -> u32 {
            self.credentials.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.credentials
                .borrow()
                .get(position as usize)
                .map(|c| c.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct CredentialList(ObjectSubclass<imp::CredentialList>)
        @implements ListModel;
}

impl CredentialList {
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Replace all credentials
    pub fn set_credentials(&self, credentials: Vec<Credential>) {
        let imp = self.imp();
        let old_len = imp.credentials.borrow().len() as u32;

        // Create new credential objects
        let new_objects: Vec<CredentialObject> =
            credentials.iter().map(CredentialObject::new).collect();
        let new_len = new_objects.len() as u32;

        imp.credentials.replace(new_objects);

        // Notify the list model of the change
        self.items_changed(0, old_len, new_len);
    }

    /// Update a specific credential's code
    pub fn update_code(&self, id: &CredentialId, code: Option<String>) {
        let credentials = self.imp().credentials.borrow();
        for cred in credentials.iter() {
            if cred.id() == *id {
                cred.update_code(code);
                // The credential object notifies its own properties
                break;
            }
        }
    }

    /// Update all TOTP credentials (mark codes as needing refresh)
    pub fn clear_totp_codes(&self) {
        let credentials = self.imp().credentials.borrow();
        for cred in credentials.iter() {
            if cred.is_totp() && !cred.touch_required() {
                cred.update_code(None);
            }
        }
    }

    /// Find a credential by ID
    pub fn find_by_id(&self, id: &CredentialId) -> Option<CredentialObject> {
        self.imp()
            .credentials
            .borrow()
            .iter()
            .find(|c| c.id() == *id)
            .cloned()
    }

    /// Remove a credential by ID
    pub fn remove_by_id(&self, id: &CredentialId) {
        let mut credentials = self.imp().credentials.borrow_mut();
        if let Some(pos) = credentials.iter().position(|c| c.id() == *id) {
            credentials.remove(pos);
            drop(credentials); // Release borrow before emitting signal
            self.items_changed(pos as u32, 1, 0);
        }
    }

    /// Add a new credential
    pub fn add(&self, credential: &Credential) {
        let mut credentials = self.imp().credentials.borrow_mut();
        let pos = credentials.len() as u32;
        credentials.push(CredentialObject::new(credential));
        drop(credentials);
        self.items_changed(pos, 0, 1);
    }

    /// Get all credentials
    pub fn all(&self) -> Vec<CredentialObject> {
        self.imp().credentials.borrow().clone()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.imp().credentials.borrow().is_empty()
    }

    /// Get count
    pub fn len(&self) -> usize {
        self.imp().credentials.borrow().len()
    }
}

impl Default for CredentialList {
    fn default() -> Self {
        Self::new()
    }
}
