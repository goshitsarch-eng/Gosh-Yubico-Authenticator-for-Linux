use crate::app::{should_clear_clipboard, Event, Outcome};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

enum Request {
    Copy(Zeroizing<String>, Duration),
    Clear,
    Stop(async_channel::Sender<()>),
}

/// Owns native clipboard access and expiry independently of blocking PC/SC calls.
#[derive(Clone)]
pub struct ClipboardService {
    sender: mpsc::SyncSender<Request>,
}
impl ClipboardService {
    pub fn start(events: async_channel::Sender<Event>) -> Self {
        let (sender, receiver) = mpsc::sync_channel(16);
        std::thread::spawn(move || {
            let mut clipboard = None;
            let mut owned: Option<(Zeroizing<String>, Instant)> = None;
            loop {
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(Request::Copy(text, timeout)) => {
                        let result = (|| {
                            if clipboard.is_none() {
                                clipboard = Some(arboard::Clipboard::new()?);
                            }
                            if let Some(clipboard) = clipboard.as_mut() {
                                clipboard.set_text(text.as_str())?;
                            }
                            Ok::<_, arboard::Error>(())
                        })();
                        match result {
                            Ok(()) => {
                                owned = Some((text, Instant::now() + timeout));
                                let _ = events.try_send(Event::Success(Outcome::Copied));
                            }
                            Err(e) => {
                                let _ = events
                                    .try_send(Event::Error(format!("Clipboard unavailable: {e}")));
                            }
                        }
                    }
                    Ok(Request::Clear) => clear(&mut clipboard, &mut owned, &events),
                    Ok(Request::Stop(done)) => {
                        clear(&mut clipboard, &mut owned, &events);
                        let _ = done.try_send(());
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        clear(&mut clipboard, &mut owned, &events);
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if owned
                    .as_ref()
                    .is_some_and(|(_, deadline)| Instant::now() >= *deadline)
                {
                    clear(&mut clipboard, &mut owned, &events);
                }
            }
        });
        Self { sender }
    }
    pub fn copy(&self, text: String, timeout: Duration) -> Result<(), String> {
        self.sender
            .try_send(Request::Copy(Zeroizing::new(text), timeout))
            .map_err(|_| "Clipboard service is busy or stopped".into())
    }
    pub fn clear(&self) {
        let _ = self.sender.try_send(Request::Clear);
    }
    pub async fn stop(&self) {
        let (done, receiver) = async_channel::bounded(1);
        // Await queue space without blocking the GUI thread.
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let _ = sender.send(Request::Stop(done));
        });
        let _ = receiver.recv().await;
    }
}
fn clear(
    clipboard: &mut Option<arboard::Clipboard>,
    owned: &mut Option<(Zeroizing<String>, Instant)>,
    events: &async_channel::Sender<Event>,
) {
    if let (Some((text, _)), Some(clipboard)) = (owned.take(), clipboard.as_mut()) {
        match clipboard.get_text() {
            Ok(current) if should_clear_clipboard(&text, &current) => {
                if let Err(e) = clipboard.clear() {
                    let _ =
                        events.try_send(Event::Error(format!("Could not clear clipboard: {e}")));
                }
            }
            Ok(_) => {}
            Err(e) => log::debug!("Clipboard ownership changed: {e}"),
        }
    }
}
