use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::subclass::prelude::*;

use crate::widgets::GoshWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GoshApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for GoshApplication {
        const NAME: &'static str = "GoshApplication";
        type Type = super::GoshApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for GoshApplication {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_actions();
            obj.setup_accels();
        }
    }

    impl ApplicationImpl for GoshApplication {
        fn startup(&self) {
            self.parent_startup();

            // Setup icon theme - must happen after GTK is initialized
            if let Some(display) = gtk4::gdk::Display::default() {
                let icon_theme = gtk4::IconTheme::for_display(&display);
                icon_theme.add_resource_path("/com/github/gosh/authenticator/icons");

                // Debug: check if icon can be found
                if icon_theme.has_icon("com.github.gosh.authenticator") {
                    log::info!("Icon found in theme!");
                } else {
                    log::warn!("Icon NOT found in theme. Search paths: {:?}", icon_theme.search_path());
                    log::warn!("Resource paths: {:?}", icon_theme.resource_path());
                }
            }

            // Set default icon for all windows
            gtk4::Window::set_default_icon_name("com.github.gosh.authenticator");
        }

        fn activate(&self) {
            let application = self.obj();

            // Get the current window or create one
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = GoshWindow::new(&*application);
                window.upcast()
            };

            window.present();
        }
    }

    impl GtkApplicationImpl for GoshApplication {}
    impl AdwApplicationImpl for GoshApplication {}
}

glib::wrapper! {
    pub struct GoshApplication(ObjectSubclass<imp::GoshApplication>)
        @extends adw::Application, gtk4::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GoshApplication {
    pub fn new(application_id: &str, flags: gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_actions(&self) {
        // Quit action
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(|app: &Self, _, _| {
                app.quit();
            })
            .build();

        // About action
        let about_action = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about();
            })
            .build();

        self.add_action_entries([quit_action, about_action]);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("window.close", &["<primary>w"]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();

        let about = adw::AboutWindow::builder()
            .application_name("Gosh Authenticator")
            .application_icon("com.github.gosh.authenticator")
            .developer_name("Gosh Team")
            .version(env!("CARGO_PKG_VERSION"))
            .website("https://github.com/gosh/gosh-yubico-authenticator")
            .issue_url("https://github.com/gosh/gosh-yubico-authenticator/issues")
            .license_type(gtk4::License::Gpl30)
            .copyright("© 2024 Gosh Team")
            .developers(vec!["Gosh Team"])
            .comments("Manage OATH credentials on YubiKey devices")
            .transient_for(&window)
            .modal(true)
            .build();

        about.present();
    }
}

impl Default for GoshApplication {
    fn default() -> Self {
        Self::new(
            "com.github.gosh.authenticator",
            gio::ApplicationFlags::empty(),
        )
    }
}
