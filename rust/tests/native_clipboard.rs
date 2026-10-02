#![cfg(feature = "desktop")]
use gosh_authenticator_core::{app::Event, services::clipboard::ClipboardService};
use std::time::Duration;

#[test]
#[ignore = "Requires an isolated native desktop clipboard; run explicitly on Xvfb/native CI"]
fn expiry_and_shutdown_preserve_newer_native_clipboard_contents() {
    let (events, receiver) = async_channel::bounded(16);
    let service = ClipboardService::start(events);
    let mut clipboard = arboard::Clipboard::new().unwrap();
    service
        .copy("123456".into(), Duration::from_millis(400))
        .unwrap();
    assert!(matches!(
        receiver.recv_blocking().unwrap(),
        Event::Success(_)
    ));
    assert_eq!(clipboard.get_text().unwrap(), "123456");
    clipboard.set_text("newer user text").unwrap();
    std::thread::sleep(Duration::from_millis(600));
    assert_eq!(clipboard.get_text().unwrap(), "newer user text");
    service
        .copy("654321".into(), Duration::from_millis(200))
        .unwrap();
    assert!(matches!(
        receiver.recv_blocking().unwrap(),
        Event::Success(_)
    ));
    std::thread::sleep(Duration::from_millis(400));
    assert_ne!(clipboard.get_text().ok().as_deref(), Some("654321"));
    service
        .copy("111111".into(), Duration::from_secs(60))
        .unwrap();
    assert!(matches!(
        receiver.recv_blocking().unwrap(),
        Event::Success(_)
    ));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(service.stop());
    assert_ne!(clipboard.get_text().ok().as_deref(), Some("111111"));
}
