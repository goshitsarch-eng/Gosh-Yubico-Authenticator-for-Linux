use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use adw::prelude::*;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use glib::prelude::ToValue;

use crate::prefs::{
    apply_theme, APP_ID, APP_NAME, APP_VERSION, CLIPBOARD_TIMEOUTS,
};
use crate::service_icons::{get_service_by_key, guess_domain, guess_service};
use gosh_authenticator_core::core::credential::Credential;
use gosh_authenticator_core::core::yubikey::OathType;
use gosh_authenticator_core::services::Event;
use crate::ui::add_page::AddPage;
use crate::ui::dialogs::{
    present_about, present_change_password_dialog, present_delete_dialog, present_icon_picker,
    present_pin_dialog, present_touch_dialog,
};
use crate::ui::state::{AppState, ConnectionStatus};

struct Widgets {
    window: adw::ApplicationWindow,
    toast: adw::ToastOverlay,
    nav: adw::NavigationView,
    stack: adw::ViewStack,
    add_button: gtk::Button,
    refresh_button: gtk::Button,
    search: gtk::SearchEntry,
    creds_stack: gtk::Stack,
    creds_status: adw::StatusPage,
    creds_list: gtk::ListBox,
    banner: adw::Banner,
    key_info_stack: gtk::Stack,
    key_info_status: adw::StatusPage,
    device_name: gtk::Label,
    firmware: gtk::Label,
    total_count: gtk::Label,
    totp_count: gtk::Label,
    hotp_count: gtk::Label,
    theme_row: adw::ComboRow,
    timeout_row: adw::ComboRow,
    require_pin_row: adw::SwitchRow,
    change_pin_row: adw::ActionRow,
    settings_status: gtk::Label,
    progress_values: RefCell<Vec<(Vec<u8>, Rc<Cell<f64>>, gtk::DrawingArea)>>,
}

pub fn build_ui(app: &adw::Application) {
    let state = Rc::new(AppState::new());
    apply_theme(state.prefs.borrow().theme_mode);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(960)
        .default_height(680)
        .build();
    window.set_icon_name(Some(APP_ID));

    let toast = adw::ToastOverlay::new();
    let nav = adw::NavigationView::new();
    let stack = adw::ViewStack::new();

    let (creds_page, creds_widgets) = build_credentials_page();
    let (key_info_page, key_info_widgets) = build_key_info_page();
    let (settings_page, settings_widgets) = build_settings_page();

    stack.add_titled_with_icon(
        &creds_page,
        Some("credentials"),
        "Credentials",
        "dialog-password-symbolic",
    );
    stack.add_titled_with_icon(
        &key_info_page,
        Some("key-info"),
        "Key Info",
        "dialog-information-symbolic",
    );
    stack.add_titled_with_icon(
        &settings_page,
        Some("settings"),
        "Settings",
        "emblem-system-symbolic",
    );

    let switcher = adw::ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();
    let switcher_bar = adw::ViewSwitcherBar::builder().stack(&stack).build();

    let add_button = gtk::Button::from_icon_name("list-add-symbolic");
    add_button.set_tooltip_text(Some("Add Credential"));
    add_button.add_css_class("suggested-action");
    add_button.set_visible(false);

    let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_button.set_tooltip_text(Some("Refresh"));
    refresh_button.set_visible(false);

    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Menu")
        .build();
    let menu = gio::Menu::new();
    menu.append(Some("About Gosh Yubikey Manager"), Some("win.about"));
    menu.append(Some("Quit"), Some("window.close"));
    menu_button.set_menu_model(Some(&menu));

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&switcher));
    header.pack_start(&refresh_button);
    header.pack_end(&menu_button);
    header.pack_end(&add_button);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.add_bottom_bar(&switcher_bar);
    toolbar.set_content(Some(&stack));

    let main_page = adw::NavigationPage::builder()
        .child(&toolbar)
        .title(APP_NAME)
        .tag("main")
        .build();
    nav.add(&main_page);
    toast.set_child(Some(&nav));
    window.set_content(Some(&toast));

    let breakpoint = adw::Breakpoint::new(
        adw::BreakpointCondition::parse("max-width: 760px").expect("valid breakpoint"),
    );
    breakpoint.add_setter(&switcher_bar, "reveal", Some(&true.to_value()));
    window.add_breakpoint(breakpoint);

    let widgets = Rc::new(Widgets {
        window: window.clone(),
        toast: toast.clone(),
        nav: nav.clone(),
        stack: stack.clone(),
        add_button: add_button.clone(),
        refresh_button: refresh_button.clone(),
        search: creds_widgets.0,
        creds_stack: creds_widgets.1,
        creds_status: creds_widgets.2,
        creds_list: creds_widgets.3,
        banner: creds_widgets.4,
        key_info_stack: key_info_widgets.0,
        key_info_status: key_info_widgets.1,
        device_name: key_info_widgets.2,
        firmware: key_info_widgets.3,
        total_count: key_info_widgets.4,
        totp_count: key_info_widgets.5,
        hotp_count: key_info_widgets.6,
        theme_row: settings_widgets.0,
        timeout_row: settings_widgets.1,
        require_pin_row: settings_widgets.2,
        change_pin_row: settings_widgets.3,
        settings_status: settings_widgets.4,
        progress_values: RefCell::new(Vec::new()),
    });

    {
        let prefs = state.prefs.borrow();
        widgets.theme_row.set_selected(prefs.theme_mode.index());
        widgets
            .timeout_row
            .set_selected(prefs.clipboard_timeout_index());
        widgets
            .require_pin_row
            .set_active(prefs.require_pin_on_launch);
    }

    let about_action = gio::SimpleAction::new("about", None);
    about_action.connect_activate({
        let window = window.clone();
        move |_, _| present_about(&window)
    });
    window.add_action(&about_action);

    widgets.theme_row.connect_selected_notify({
        let state = Rc::clone(&state);
        move |row| {
            let mode = crate::prefs::ThemeMode::from_index(row.selected());
            apply_theme(mode);
            let mut prefs = state.prefs.borrow_mut();
            prefs.theme_mode = mode;
            prefs.save();
        }
    });
    widgets.timeout_row.connect_selected_notify({
        let state = Rc::clone(&state);
        move |row| {
            let mut prefs = state.prefs.borrow_mut();
            prefs.set_clipboard_timeout_index(row.selected());
            prefs.save();
        }
    });
    widgets.require_pin_row.connect_active_notify({
        let state = Rc::clone(&state);
        move |row| {
            let mut prefs = state.prefs.borrow_mut();
            prefs.require_pin_on_launch = row.is_active();
            prefs.save();
        }
    });
    widgets.change_pin_row.connect_activated({
        let state = Rc::clone(&state);
        let window = window.clone();
        move |_| {
            if state.connection.get() == ConnectionStatus::Connected {
                present_change_password_dialog(&window, Rc::clone(&state));
            }
        }
    });

    widgets.search.connect_search_changed({
        let state = Rc::clone(&state);
        let widgets = Rc::clone(&widgets);
        move |entry| {
            *state.search.borrow_mut() = entry.text().to_string();
            refresh_credentials_view(&state, &widgets);
        }
    });

    add_button.connect_clicked({
        let state = Rc::clone(&state);
        let widgets = Rc::clone(&widgets);
        move |_| {
            if state.connection.get() != ConnectionStatus::Connected {
                return;
            }
            let add = AddPage::new(
                Rc::clone(&state),
                widgets.window.clone(),
                widgets.toast.clone(),
                widgets.nav.clone(),
            );
            widgets.nav.push(&add.page);
        }
    });
    refresh_button.connect_clicked({
        let state = Rc::clone(&state);
        move |_| state.service.refresh()
    });

    stack.connect_visible_child_notify({
        let widgets = Rc::clone(&widgets);
        let state = Rc::clone(&state);
        move |_| update_header_buttons(&state, &widgets)
    });

    let event_rx = state.service.event_receiver();
    glib::timeout_add_local(Duration::from_millis(80), {
        let state = Rc::clone(&state);
        let widgets = Rc::clone(&widgets);
        move || {
            while let Ok(event) = event_rx.try_recv() {
                handle_event(&state, &widgets, event);
            }
            tick_totp(&state, &widgets);
            glib::ControlFlow::Continue
        }
    });

    refresh_all(&state, &widgets);
    state.service.connect();
    window.present();
}

type CredsWidgets = (
    gtk::SearchEntry,
    gtk::Stack,
    adw::StatusPage,
    gtk::ListBox,
    adw::Banner,
);

fn build_credentials_page() -> (gtk::Widget, CredsWidgets) {
    let search = gtk::SearchEntry::builder()
        .placeholder_text("Search accounts...")
        .hexpand(true)
        .build();
    let banner = adw::Banner::builder()
        .title("No YubiKey")
        .revealed(true)
        .build();

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let list_clamp = adw::Clamp::builder()
        .maximum_size(720)
        .child(&list)
        .build();
    list_clamp.set_margin_top(12);
    list_clamp.set_margin_bottom(24);
    list_clamp.set_margin_start(12);
    list_clamp.set_margin_end(12);
    let list_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&list_clamp)
        .build();

    let status = adw::StatusPage::builder()
        .icon_name("media-removable-symbolic")
        .title("No YubiKey Connected")
        .description("Insert your YubiKey to view credentials")
        .build();

    let creds_stack = gtk::Stack::new();
    creds_stack.add_named(&status, Some("status"));
    creds_stack.add_named(&list_scroll, Some("list"));

    let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    search_box.set_margin_start(12);
    search_box.set_margin_end(12);
    search_box.set_margin_top(12);
    search_box.append(&search);

    let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page.append(&banner);
    page.append(&search_box);
    page.append(&creds_stack);
    (page.upcast(), (search, creds_stack, status, list, banner))
}

type KeyInfoWidgets = (
    gtk::Stack,
    adw::StatusPage,
    gtk::Label,
    gtk::Label,
    gtk::Label,
    gtk::Label,
    gtk::Label,
);

fn build_key_info_page() -> (gtk::Widget, KeyInfoWidgets) {
    let status = adw::StatusPage::builder()
        .icon_name("media-removable-symbolic")
        .title("No YubiKey Connected")
        .description("Insert your YubiKey to view device info")
        .build();

    let device_name = info_value_label();
    let firmware = info_value_label();
    let total_count = info_value_label();
    let totp_count = info_value_label();
    let hotp_count = info_value_label();

    let device_group = adw::PreferencesGroup::builder()
        .title("Device Information")
        .build();
    device_group.add(&info_row("Device Name", &device_name));
    device_group.add(&info_row("Firmware Version", &firmware));

    let oath_group = adw::PreferencesGroup::builder()
        .title("OATH Credentials")
        .build();
    oath_group.add(&info_row("Total", &total_count));
    oath_group.add(&info_row("TOTP", &totp_count));
    oath_group.add(&info_row("HOTP", &hotp_count));

    let status_group = adw::PreferencesGroup::builder()
        .title("Connection Status")
        .build();
    let connected = adw::ActionRow::builder()
        .title("Connected via USB")
        .subtitle("PC/SC smart card reader")
        .build();
    connected.add_prefix(&gtk::Image::from_icon_name("emblem-ok-symbolic"));
    status_group.add(&connected);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content.set_margin_top(18);
    content.set_margin_bottom(24);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.append(&device_group);
    content.append(&oath_group);
    content.append(&status_group);
    let clamp = adw::Clamp::builder()
        .maximum_size(640)
        .child(&content)
        .build();
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build();

    let stack = gtk::Stack::new();
    stack.add_named(&status, Some("status"));
    stack.add_named(&scroll, Some("info"));
    (
        stack.clone().upcast(),
        (
            stack,
            status,
            device_name,
            firmware,
            total_count,
            totp_count,
            hotp_count,
        ),
    )
}

type SettingsWidgets = (
    adw::ComboRow,
    adw::ComboRow,
    adw::SwitchRow,
    adw::ActionRow,
    gtk::Label,
);

fn build_settings_page() -> (gtk::Widget, SettingsWidgets) {
    let theme_row = adw::ComboRow::builder()
        .title("Theme")
        .subtitle("App appearance")
        .model(&gtk::StringList::new(&["System", "Light", "Dark"]))
        .build();
    theme_row.add_prefix(&gtk::Image::from_icon_name("weather-clear-symbolic"));

    let timeout_labels: Vec<String> = CLIPBOARD_TIMEOUTS
        .iter()
        .map(|v| format!("{v}s"))
        .collect();
    let timeout_refs: Vec<&str> = timeout_labels.iter().map(String::as_str).collect();
    let timeout_row = adw::ComboRow::builder()
        .title("Clear Clipboard")
        .subtitle("Sensitive data timeout")
        .model(&gtk::StringList::new(&timeout_refs))
        .build();
    timeout_row.add_prefix(&gtk::Image::from_icon_name("edit-copy-symbolic"));

    let require_pin_row = adw::SwitchRow::builder()
        .title("Require PIN")
        .subtitle("On application launch")
        .build();
    require_pin_row.add_prefix(&gtk::Image::from_icon_name("system-lock-screen-symbolic"));

    let change_pin_row = adw::ActionRow::builder()
        .title("Change YubiKey PIN")
        .subtitle("Connect YubiKey first")
        .activatable(true)
        .build();
    change_pin_row.add_prefix(&gtk::Image::from_icon_name("dialog-password-symbolic"));
    change_pin_row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));

    let general = adw::PreferencesGroup::builder().title("General").build();
    general.add(&theme_row);

    let security = adw::PreferencesGroup::builder().title("Security").build();
    security.add(&change_pin_row);
    security.add(&require_pin_row);

    let prefs = adw::PreferencesGroup::builder()
        .title("App Preferences")
        .build();
    prefs.add(&timeout_row);

    let footer = gtk::Label::builder()
        .label(format!("{APP_NAME}  ·  Version {APP_VERSION}"))
        .css_classes(["dim-label"])
        .margin_top(24)
        .build();

    let page = adw::PreferencesPage::new();
    page.add(&general);
    page.add(&security);
    page.add(&prefs);
    page.add(
        &adw::PreferencesGroup::builder()
            .header_suffix(&footer)
            .build(),
    );

    (page.upcast(), (theme_row, timeout_row, require_pin_row, change_pin_row, footer))
}

fn info_value_label() -> gtk::Label {
    gtk::Label::builder()
        .xalign(1.0)
        .css_classes(["dim-label"])
        .selectable(true)
        .build()
}

fn info_row(title: &str, value: &gtk::Label) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(title).build();
    row.add_suffix(value);
    row
}

fn handle_event(state: &Rc<AppState>, widgets: &Rc<Widgets>, event: Event) {
    match event {
        Event::Ready => {}
        Event::Connected { version, device_id } => {
            state.connection.set(ConnectionStatus::Connected);
            state.version.set(version);
            state.device_id.set(device_id);
            state.has_device.set(true);
            widgets
                .toast
                .add_toast(adw::Toast::new("YubiKey connected"));
        }
        Event::Disconnected => {
            state.connection.set(ConnectionStatus::Disconnected);
            state.has_device.set(false);
            state.credentials.borrow_mut().clear();
        }
        Event::AuthenticationRequired => {
            state.connection.set(ConnectionStatus::Connected);
            state.has_device.set(true);
            present_pin_dialog(&widgets.window, Rc::clone(state), None, || {});
        }
        Event::AuthenticationSuccessful => {
            widgets.toast.add_toast(adw::Toast::new("YubiKey unlocked"));
        }
        Event::AuthenticationFailed(msg) => {
            present_pin_dialog(
                &widgets.window,
                Rc::clone(state),
                Some(if msg.is_empty() { "Incorrect PIN" } else { &msg }),
                || {},
            );
        }
        Event::CredentialsUpdated(credentials) => {
            *state.credentials.borrow_mut() = credentials;
            maybe_hide_touch(state, widgets);
        }
        Event::CredentialCalculated { id, code, digits } => {
            if let Some(cred) = state
                .credentials
                .borrow_mut()
                .iter_mut()
                .find(|c| c.id == id)
            {
                cred.digits = digits;
                cred.code = Some(code);
            }
            state.touch_open.set(false);
        }
        Event::TouchRequired(_) => {
            present_touch_dialog(&widgets.window, Rc::clone(state));
        }
        Event::CredentialAdded(_) => {
            widgets
                .toast
                .add_toast(adw::Toast::new("Credential added"));
        }
        Event::CredentialDeleted(_) => {
            widgets
                .toast
                .add_toast(adw::Toast::new("Credential deleted"));
        }
        Event::PasswordChanged => {
            widgets
                .toast
                .add_toast(adw::Toast::new("YubiKey PIN changed"));
        }
        Event::PasswordRemoved => {
            widgets
                .toast
                .add_toast(adw::Toast::new("Password protection removed"));
        }
        Event::Error(msg) => {
            widgets.toast.add_toast(adw::Toast::new(&msg));
        }
    }
    refresh_all(state, widgets);
}

fn maybe_hide_touch(state: &Rc<AppState>, _widgets: &Rc<Widgets>) {
    let _ = state;
}

fn tick_totp(state: &Rc<AppState>, widgets: &Rc<Widgets>) {
    if let Some((_, until)) = state.copied_until.borrow().as_ref() {
        if Instant::now() > *until {
            *state.copied_until.borrow_mut() = None;
            refresh_credentials_view(state, widgets);
        }
    }

    if state.connection.get() == ConnectionStatus::Connected {
        let slot = AppState::period_slot(30);
        if state.last_period_slot.get() == 0 {
            state.last_period_slot.set(slot);
        } else if slot != state.last_period_slot.get() {
            state.last_period_slot.set(slot);
            state.service.refresh();
        }
    }

    for (id, value, area) in widgets.progress_values.borrow().iter() {
        let period = state
            .credentials
            .borrow()
            .iter()
            .find(|c| c.id.0 == *id)
            .map(|c| c.period)
            .unwrap_or(30);
        value.set(AppState::totp_progress(period));
        area.queue_draw();
    }
}

fn refresh_all(state: &Rc<AppState>, widgets: &Rc<Widgets>) {
    update_header_buttons(state, widgets);
    refresh_credentials_view(state, widgets);
    refresh_key_info(state, widgets);
    refresh_settings(state, widgets);
}

fn update_header_buttons(state: &Rc<AppState>, widgets: &Rc<Widgets>) {
    let connected = state.connection.get() == ConnectionStatus::Connected;
    let on_creds = widgets
        .stack
        .visible_child_name()
        .map(|n| n == "credentials")
        .unwrap_or(true);
    widgets.add_button.set_visible(connected && on_creds);
    widgets.refresh_button.set_visible(connected);
}

fn refresh_credentials_view(state: &Rc<AppState>, widgets: &Rc<Widgets>) {
    match state.connection.get() {
        ConnectionStatus::Disconnected | ConnectionStatus::Connecting => {
            widgets.banner.set_title("No YubiKey");
            widgets.banner.set_revealed(true);
            widgets.creds_status.set_icon_name(Some("media-removable-symbolic"));
            widgets.creds_status.set_title("No YubiKey Connected");
            widgets
                .creds_status
                .set_description(Some("Insert your YubiKey to view credentials"));
            let retry = gtk::Button::builder()
                .label("Retry")
                .css_classes(["suggested-action", "pill"])
                .halign(gtk::Align::Center)
                .build();
            retry.connect_clicked({
                let state = Rc::clone(state);
                move |_| {
                    state.connection.set(ConnectionStatus::Connecting);
                    state.service.connect();
                }
            });
            widgets.creds_status.set_child(Some(&retry));
            widgets.creds_stack.set_visible_child_name("status");
            widgets.progress_values.borrow_mut().clear();
            return;
        }
        ConnectionStatus::Connected => {
            widgets.banner.set_title("Connected via USB");
            widgets.banner.set_revealed(true);
        }
    }

    let filtered = state.filtered_credentials();
    if filtered.is_empty() {
        widgets.creds_status.set_icon_name(Some("dialog-password-symbolic"));
        widgets.creds_status.set_title("No Credentials");
        widgets
            .creds_status
            .set_description(Some("Add your first credential to get started"));
        let add = gtk::Button::builder()
            .label("Add Credential")
            .css_classes(["suggested-action", "pill"])
            .halign(gtk::Align::Center)
            .build();
        add.connect_clicked({
            let state = Rc::clone(state);
            let widgets = Rc::clone(widgets);
            move |_| {
                let add = AddPage::new(
                    Rc::clone(&state),
                    widgets.window.clone(),
                    widgets.toast.clone(),
                    widgets.nav.clone(),
                );
                widgets.nav.push(&add.page);
            }
        });
        widgets.creds_status.set_child(Some(&add));
        widgets.creds_stack.set_visible_child_name("status");
        widgets.progress_values.borrow_mut().clear();
        return;
    }

    while let Some(child) = widgets.creds_list.first_child() {
        widgets.creds_list.remove(&child);
    }
    widgets.progress_values.borrow_mut().clear();

    for cred in filtered {
        let row = build_credential_row(state, widgets, &cred);
        widgets.creds_list.append(&row);
    }
    widgets.creds_stack.set_visible_child_name("list");
}

fn build_credential_row(
    state: &Rc<AppState>,
    widgets: &Rc<Widgets>,
    cred: &Credential,
) -> gtk::ListBoxRow {
    let title = cred
        .issuer
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| cred.account.clone());
    let subtitle = if cred.issuer.as_ref().is_some_and(|s| !s.is_empty()) {
        Some(cred.account.clone())
    } else {
        None
    };

    let title_label = gtk::Label::builder()
        .label(&title)
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["credential-title"])
        .build();
    let subtitle_label = gtk::Label::builder()
        .label(subtitle.as_deref().unwrap_or(""))
        .xalign(0.0)
        .visible(subtitle.is_some())
        .css_classes(["dim-label", "caption"])
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let text_col = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_col.set_hexpand(true);
    text_col.set_valign(gtk::Align::Center);
    text_col.append(&title_label);
    text_col.append(&subtitle_label);
    if cred.touch_required && cred.code.is_none() {
        let badge = gtk::Label::builder()
            .label("TOUCH REQUIRED")
            .xalign(0.0)
            .css_classes(["touch-required"])
            .build();
        text_col.append(&badge);
    }

    let copied = state
        .copied_until
        .borrow()
        .as_ref()
        .is_some_and(|(id, until)| *id == cred.id && Instant::now() <= *until);

    let code_label = gtk::Label::builder()
        .xalign(1.0)
        .selectable(false)
        .build();
    if copied {
        code_label.set_label("COPIED");
        code_label.add_css_class("copied-label");
    } else if let Some(code) = &cred.code {
        code_label.set_label(code);
        code_label.add_css_class("otp-code");
        if cred.is_totp() && AppState::totp_progress(cred.period) <= 0.25 {
            code_label.add_css_class("expiring");
        }
    } else {
        code_label.set_label("• • •   • • •");
        code_label.add_css_class("dim-label");
    }

    let progress = gtk::DrawingArea::builder()
        .content_width(28)
        .content_height(28)
        .valign(gtk::Align::Center)
        .build();
    let progress_value = Rc::new(Cell::new(AppState::totp_progress(cred.period)));
    progress.set_draw_func({
        let progress_value = Rc::clone(&progress_value);
        move |_, cr, w, h| {
            let value = progress_value.get().clamp(0.0, 1.0);
            let cx = w as f64 / 2.0;
            let cy = h as f64 / 2.0;
            let radius = (w.min(h) as f64 / 2.0) - 2.5;
            cr.set_line_width(3.0);
            cr.set_source_rgba(0.5, 0.5, 0.5, 0.25);
            cr.arc(cx, cy, radius, 0.0, std::f64::consts::TAU);
            let _ = cr.stroke();
            if value <= 0.25 {
                cr.set_source_rgb(0.95, 0.62, 0.18);
            } else {
                cr.set_source_rgb(0.21, 0.52, 0.89);
            }
            cr.arc(
                cx,
                cy,
                radius,
                -std::f64::consts::FRAC_PI_2,
                -std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU * value,
            );
            let _ = cr.stroke();
        }
    });
    if cred.is_totp() && cred.code.is_some() {
        widgets.progress_values.borrow_mut().push((
            cred.id.0.clone(),
            Rc::clone(&progress_value),
            progress.clone(),
        ));
    }

    let trailing = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    trailing.set_valign(gtk::Align::Center);
    trailing.append(&code_label);
    if cred.is_hotp() {
        let hotp = gtk::Label::builder()
            .label("HOTP")
            .css_classes(["hotp-label"])
            .build();
        trailing.append(&hotp);
    } else if cred.code.is_some() {
        trailing.append(&progress);
    } else {
        let calc = gtk::Button::from_icon_name("media-playback-start-symbolic");
        calc.set_tooltip_text(Some("Calculate"));
        calc.add_css_class("flat");
        calc.connect_clicked({
            let state = Rc::clone(state);
            let id = cred.id.clone();
            move |_| state.service.calculate(id.clone())
        });
        trailing.append(&calc);
    }

    let avatar = build_avatar(state, cred);

    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row_box.set_margin_top(10);
    row_box.set_margin_bottom(10);
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);
    row_box.append(&avatar);
    row_box.append(&text_col);
    row_box.append(&trailing);

    let row = gtk::ListBoxRow::builder().activatable(true).child(&row_box).build();
    let id = cred.id.clone();
    let display = cred.display_name();
    let has_code = cred.code.clone();
    let click = gtk::GestureClick::new();
    click.set_button(1);
    click.connect_released({
        let state = Rc::clone(state);
        let widgets = Rc::clone(widgets);
        let id = id.clone();
        let has_code = has_code.clone();
        move |_, _, _, _| {
            if let Some(code) = &has_code {
                let timeout = state.prefs.borrow().clipboard_timeout_seconds;
                state.clipboard.copy(code, timeout);
                *state.copied_until.borrow_mut() =
                    Some((id.clone(), Instant::now() + Duration::from_secs(2)));
                widgets.toast.add_toast(adw::Toast::new(&format!(
                    "Code copied (clears in {timeout}s)"
                )));
                refresh_credentials_view(&state, &widgets);
            } else {
                state.service.calculate(id.clone());
            }
        }
    });
    row.add_controller(click);

    let gesture = gtk::GestureClick::new();
    gesture.set_button(3);
    gesture.connect_released({
        let state = Rc::clone(state);
        let widgets = Rc::clone(widgets);
        let id = id.clone();
        let display = display.clone();
        let has_code = has_code.clone();
        let domain = guess_domain(cred.issuer.as_deref(), &cred.account);
        move |g, _, x, y| {
            let menu = gio::Menu::new();
            if has_code.is_some() {
                menu.append(Some("Copy code"), Some("row.copy"));
            }
            menu.append(Some("Calculate"), Some("row.calculate"));
            menu.append(Some("Choose icon"), Some("row.icon"));
            menu.append(Some("Delete"), Some("row.delete"));
            let popover = gtk::PopoverMenu::from_model(Some(&menu));
            popover.set_parent(&g.widget().expect("row"));
            popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));

            let group = gio::SimpleActionGroup::new();
            let copy = gio::SimpleAction::new("copy", None);
            copy.connect_activate({
                let state = Rc::clone(&state);
                let widgets = Rc::clone(&widgets);
                let id = id.clone();
                let has_code = has_code.clone();
                move |_, _| {
                    if let Some(code) = &has_code {
                        let timeout = state.prefs.borrow().clipboard_timeout_seconds;
                        state.clipboard.copy(code, timeout);
                        *state.copied_until.borrow_mut() =
                            Some((id.clone(), Instant::now() + Duration::from_secs(2)));
                        widgets.toast.add_toast(adw::Toast::new(&format!(
                            "Code copied (clears in {timeout}s)"
                        )));
                        refresh_credentials_view(&state, &widgets);
                    }
                }
            });
            let calculate = gio::SimpleAction::new("calculate", None);
            calculate.connect_activate({
                let state = Rc::clone(&state);
                let id = id.clone();
                move |_, _| state.service.calculate(id.clone())
            });
            let icon = gio::SimpleAction::new("icon", None);
            icon.connect_activate({
                let state = Rc::clone(&state);
                let widgets = Rc::clone(&widgets);
                let id = id.clone();
                let domain = domain.clone();
                move |_, _| {
                    present_icon_picker(
                        &widgets.window,
                        Rc::clone(&state),
                        id.0.clone(),
                        domain.clone(),
                        {
                            let state = Rc::clone(&state);
                            let widgets = Rc::clone(&widgets);
                            move || refresh_credentials_view(&state, &widgets)
                        },
                    );
                }
            });
            let delete = gio::SimpleAction::new("delete", None);
            delete.connect_activate({
                let state = Rc::clone(&state);
                let widgets = Rc::clone(&widgets);
                let id = id.clone();
                let display = display.clone();
                move |_, _| {
                    present_delete_dialog(
                        &widgets.window,
                        Rc::clone(&state),
                        display.clone(),
                        id.clone(),
                    );
                }
            });
            group.add_action(&copy);
            group.add_action(&calculate);
            group.add_action(&icon);
            group.add_action(&delete);
            g.widget().unwrap().insert_action_group("row", Some(&group));
            popover.popup();
        }
    });
    row.add_controller(gesture);
    row
}

fn build_avatar(state: &Rc<AppState>, cred: &Credential) -> gtk::Widget {
    let key = crate::prefs::Prefs::icon_key(&cred.id.0);
    let pref = state.prefs.borrow().icon_prefs.get(&key).cloned();
    let service = pref
        .as_ref()
        .and_then(|p| p.custom_icon_key.as_deref())
        .and_then(get_service_by_key)
        .or_else(|| guess_service(cred.issuer.as_deref(), &cred.account));

    let (letter, color) = if let Some(service) = service {
        (
            service
                .label
                .chars()
                .next()
                .unwrap_or('?')
                .to_ascii_uppercase()
                .to_string(),
            service.color,
        )
    } else {
        let source = cred
            .issuer
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(cred.account.as_str());
        let letter = source
            .chars()
            .find(|c| c.is_alphanumeric())
            .unwrap_or('?')
            .to_ascii_uppercase()
            .to_string();
        (letter, (0x35, 0x84, 0xE4))
    };

    let overlay = gtk::Overlay::new();
    let color_box = gtk::Box::builder()
        .width_request(36)
        .height_request(36)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    let class_name = format!("avatar-bg-{}-{}-{}", color.0, color.1, color.2);
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&format!(
        ".{class_name} {{ background-color: rgb({},{},{}); border-radius: 18px; min-width: 36px; min-height: 36px; }}",
        color.0, color.1, color.2
    ));
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    color_box.add_css_class(&class_name);

    let label = gtk::Label::builder()
        .label(&letter)
        .css_classes(["avatar-letter"])
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    overlay.set_child(Some(&color_box));
    overlay.add_overlay(&label);

    if let Some(domain) = pref
        .as_ref()
        .and_then(|p| p.favicon_domain.clone())
        .or_else(|| guess_domain(cred.issuer.as_deref(), &cred.account))
    {
        if let Some(path) = cached_favicon_path(state, &domain) {
            let picture = gtk::Image::from_file(path.to_string_lossy().as_ref());
            picture.set_pixel_size(28);
            return picture.upcast();
        } else {
            spawn_favicon_fetch(Rc::clone(state), domain);
        }
    }

    overlay.upcast()
}

fn cached_favicon_path(_state: &Rc<AppState>, domain: &str) -> Option<std::path::PathBuf> {
    let path = crate::prefs::cache_dir()
        .join("favicons")
        .join(format!("{domain}.png"));
    if !path.exists() {
        return None;
    }
    if let Ok(meta) = std::fs::metadata(&path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = std::time::SystemTime::now().duration_since(modified) {
                if age.as_secs() > 7 * 24 * 60 * 60 {
                    return None;
                }
            }
        }
    }
    Some(path)
}

fn spawn_favicon_fetch(_state: Rc<AppState>, domain: String) {
    std::thread::spawn(move || {
        let url = format!("https://www.google.com/s2/favicons?domain={domain}&sz=64");
        let Ok(response) = ureq::get(&url).timeout(Duration::from_secs(10)).call() else {
            return;
        };
        let mut bytes = Vec::new();
        if response.into_reader().read_to_end(&mut bytes).is_err() || bytes.len() <= 100 {
            return;
        }
        let dir = crate::prefs::cache_dir().join("favicons");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("{domain}.png"));
        let _ = std::fs::write(path, bytes);
    });
}

fn refresh_key_info(state: &Rc<AppState>, widgets: &Rc<Widgets>) {
    if state.connection.get() != ConnectionStatus::Connected || !state.has_device.get() {
        widgets.key_info_stack.set_visible_child_name("status");
        return;
    }
    let (major, minor, patch) = state.version.get();
    widgets
        .device_name
        .set_label(&format!("YubiKey {major}"));
    widgets
        .firmware
        .set_label(&format!("{major}.{minor}.{patch}"));
    let creds = state.credentials.borrow();
    widgets.total_count.set_label(&creds.len().to_string());
    widgets
        .totp_count
        .set_label(&creds.iter().filter(|c| c.oath_type == OathType::Totp).count().to_string());
    widgets
        .hotp_count
        .set_label(&creds.iter().filter(|c| c.oath_type == OathType::Hotp).count().to_string());
    widgets.key_info_stack.set_visible_child_name("info");
}

fn refresh_settings(state: &Rc<AppState>, widgets: &Rc<Widgets>) {
    let connected = state.connection.get() == ConnectionStatus::Connected;
    widgets.change_pin_row.set_sensitive(connected);
    widgets.change_pin_row.set_subtitle(if connected {
        "Change the OATH password"
    } else {
        "Connect YubiKey first"
    });
    widgets.settings_status.set_label(&format!(
        "{APP_NAME}  ·  Version {APP_VERSION}{}",
        if connected { "  ·  Connected" } else { "" }
    ));
}
