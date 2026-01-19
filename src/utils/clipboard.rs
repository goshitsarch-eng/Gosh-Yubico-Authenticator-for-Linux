use gtk4::gdk;
use gtk4::prelude::*;

/// Copy text to clipboard
pub fn copy_to_clipboard(display: &gdk::Display, text: &str) {
    let clipboard = display.clipboard();
    clipboard.set_text(text);
}

/// Copy text to clipboard with a timeout (auto-clear after specified seconds)
/// Note: This is a basic implementation. For security-sensitive use cases,
/// consider implementing a proper timed clipboard clear mechanism.
pub fn copy_to_clipboard_timed(display: &gdk::Display, text: &str, _timeout_seconds: u32) {
    // Basic copy - timeout clearing would need additional implementation
    copy_to_clipboard(display, text);
}
