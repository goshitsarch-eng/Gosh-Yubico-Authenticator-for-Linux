use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;

/// Copy text to the clipboard and clear it later if it is still unchanged.
pub struct ClipboardService {
    last_copied: Rc<RefCell<Option<String>>>,
    timeout_source: Rc<RefCell<Option<glib::SourceId>>>,
}

impl ClipboardService {
    pub fn new() -> Self {
        Self {
            last_copied: Rc::new(RefCell::new(None)),
            timeout_source: Rc::new(RefCell::new(None)),
        }
    }

    pub fn copy(&self, text: &str, timeout_seconds: u32) {
        let Some(display) = gdk::Display::default() else {
            return;
        };
        let clipboard = display.clipboard();
        clipboard.set_text(text);
        *self.last_copied.borrow_mut() = Some(text.to_string());

        if let Some(id) = self.timeout_source.borrow_mut().take() {
            id.remove();
        }

        let last_copied = Rc::clone(&self.last_copied);
        let timeout_source = Rc::clone(&self.timeout_source);
        let copied = text.to_string();
        let id = glib::timeout_add_local(
            Duration::from_secs(timeout_seconds as u64),
            move || {
                *timeout_source.borrow_mut() = None;
                if last_copied.borrow().as_deref() == Some(copied.as_str()) {
                    if let Some(display) = gdk::Display::default() {
                        let clipboard = display.clipboard();
                        clipboard.set_text("");
                    }
                    *last_copied.borrow_mut() = None;
                }
                glib::ControlFlow::Break
            },
        );
        *self.timeout_source.borrow_mut() = Some(id);
    }
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new()
    }
}
