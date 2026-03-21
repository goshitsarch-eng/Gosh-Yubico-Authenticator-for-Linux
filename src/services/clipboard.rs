use arboard::Clipboard;
use std::sync::Mutex;
use std::time::Duration;
use std::thread;

static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

/// Copy text to the system clipboard, optionally clearing after a timeout
pub fn copy_to_clipboard(text: &str, clear_after_secs: Option<u64>) {
    let mut guard = CLIPBOARD.lock().unwrap();
    let clipboard = guard.get_or_insert_with(|| {
        Clipboard::new().expect("Failed to initialize clipboard")
    });

    if let Err(e) = clipboard.set_text(text) {
        log::error!("Failed to copy to clipboard: {}", e);
        return;
    }

    if let Some(secs) = clear_after_secs {
        let text_owned = text.to_string();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(secs));
            let mut guard = CLIPBOARD.lock().unwrap();
            if let Some(clipboard) = guard.as_mut() {
                // Only clear if it still contains our text
                if let Ok(current) = clipboard.get_text() {
                    if current == text_owned {
                        let _ = clipboard.set_text("");
                    }
                }
            }
        });
    }
}
