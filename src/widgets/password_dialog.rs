use glib::clone;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct PasswordDialog {
        pub password_entry: RefCell<Option<adw::PasswordEntryRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PasswordDialog {
        const NAME: &'static str = "GoshPasswordDialog";
        type Type = super::PasswordDialog;
        type ParentType = adw::MessageDialog;
    }

    impl ObjectImpl for PasswordDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_heading(Some("YubiKey Password Required"));
            obj.set_body("This YubiKey is password protected. Enter the password to access OATH credentials.");

            // Create password entry
            let entry = adw::PasswordEntryRow::builder()
                .title("Password")
                .build();

            // Add the entry as extra child
            let clamp = adw::Clamp::builder()
                .maximum_size(300)
                .margin_top(12)
                .child(&entry)
                .build();

            obj.set_extra_child(Some(&clamp));
            self.password_entry.replace(Some(entry));

            // Add responses
            obj.add_response("cancel", "Cancel");
            obj.add_response("unlock", "Unlock");
            obj.set_response_appearance("unlock", adw::ResponseAppearance::Suggested);
            obj.set_default_response(Some("unlock"));
            obj.set_close_response("cancel");
        }
    }

    impl WidgetImpl for PasswordDialog {}
    impl WindowImpl for PasswordDialog {}
    impl MessageDialogImpl for PasswordDialog {}
}

glib::wrapper! {
    pub struct PasswordDialog(ObjectSubclass<imp::PasswordDialog>)
        @extends adw::MessageDialog, gtk4::Window, gtk4::Widget;
}

impl PasswordDialog {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn password(&self) -> String {
        self.imp()
            .password_entry
            .borrow()
            .as_ref()
            .map(|e| e.text().to_string())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Some(entry) = self.imp().password_entry.borrow().as_ref() {
            entry.set_text("");
        }
    }

    /// Show the dialog and get the password if user clicks "Unlock"
    pub async fn run(&self, parent: &impl IsA<gtk4::Widget>) -> Option<String> {
        if let Some(window) = parent.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
            self.set_transient_for(Some(&window));
        }
        self.set_modal(true);

        let (sender, receiver) = async_channel::bounded::<Option<String>>(1);

        self.connect_response(
            None,
            clone!(
                #[weak(rename_to = dialog)]
                self,
                #[strong]
                sender,
                move |_, response| {
                    let result = if response == "unlock" {
                        Some(dialog.password())
                    } else {
                        None
                    };
                    let _ = sender.send_blocking(result);
                }
            ),
        );

        self.present();

        receiver.recv().await.ok().flatten()
    }
}

impl Default for PasswordDialog {
    fn default() -> Self {
        Self::new()
    }
}
