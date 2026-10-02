# Architecture

The migration targets one Rust application with a Dioxus Desktop frontend. The
OATH credential store remains the hardware key; no local database stores OTP
secrets. `rust/src/core` contains protocol/domain behavior independent of UI.
`services` owns blocking smart-card work on one dedicated worker. `app` reduces
events into explicit connection/credential/presentation state and routes shared
commands. `settings` owns validated, backward-compatible JSON and atomic writes.
`services/clipboard.rs` owns native clipboard access and independent expiry.
`platform` owns native dialogs, URL opening, native menu conventions
and window integration. `ui` contains reusable Dioxus components and CSS.

UI entry points send the same commands; neither components nor native menus
perform OATH/crypto/filesystem operations. Device I/O is serialized off the UI
thread. Events distinguish disconnected, locked, ready, busy, touch required and
recoverable errors. Detected device removal clears calculated codes; presence is polled every two seconds. TOTP
refresh follows each credential's period; HOTP advances only on Generate.

Persistence retains `gosh-authenticator/settings.json`, the old field names and
Base64 icon IDs, using standard per-platform configuration directories. New
fields have serde defaults. Unreadable or malformed files produce an explicit
error and are preserved. Writes use an atomic replacement, not partial overwrite.
Settings contain preferences and window geometry; passwords/OTP secrets/codes
remain transient and are excluded from logs. Password/key buffers use zeroizing
storage where practical.

Dioxus uses OS WebViews through Dioxus Desktop/Wry: WebView2 on Windows, WKWebView
on macOS and WebKitGTK on Linux. GTK on Linux is an implementation dependency of
the WebView/window integration, not the application frontend. There is no Node,
Electron, React or TypeScript application runtime. CSS and assets are embedded,
so no remote web application or network content is loaded into the UI. File
dialogs use native APIs/desktop portals. PC/SC uses WinSCard, the macOS PCSC
framework or pcsc-lite. Remaining OS-specific code belongs in `platform`.

The version is `2.0.0-alpha.1` until the migration inventory, hardware checks,
Windows/macOS native QA and Flatpak sandbox QA have evidence. Platform support
documentation must report implementation, compilation and runtime evidence
separately. CI builds/tests on each operating system; release packaging must not
silently treat missing artifacts as successful output.
