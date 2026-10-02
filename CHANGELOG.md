# Changelog

## 2.0.0-alpha.1 — migration in progress, 2026-10-01

- Replace the Linux-only GTK 4 frontend with one Rust/Dioxus Desktop application.
- Separate domain/protocol code, background services, state, persistence and native integration.
- Fix Yubico authentication key derivation, SET_CODE type, touch encoding, strict TLV parsing,
  secret shortening, duplicate overwrite protection, HOTP classification/counters and custom periods.
- Preserve legacy settings and unknown fields; save atomically and surface errors.
- Bound and offload QR parsing; preserve Unicode, counters and periods from OTP URIs.
- Isolate clipboard ownership/expiry from device waits; preserve newer user text.
- Add native menus/dialogs, Ctrl/Cmd commands, form focus, window state and responsive themes.
- Add opt-in, validated favicon requests and working cached-icon events.
- Add native platform CI, MSI/app/Linux packaging, both Flatpak architectures and guarded draft releases.
- Fix the Rustls advisory and replace the QR decoder's affected lru dependency.
- Native Windows/macOS, real-key, Wayland and signing/notarization validation remain QA gates.

Historical 1.x implementation and release notes remain in Git history.
