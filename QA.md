# Migration QA record

Baseline audit began 2026-10-01; migration verification continues 2026-10-02.
No physical YubiKey is available. This record will distinguish completed checks
from required checks instead of presenting source inspection as runtime proof.

## Observed locally

- Baseline GTK application built, launched and audited before source changes.
- Dioxus Desktop application built in debug and optimized release modes.
- Rust software tests exercise APDU/TLV parsing, Yubico authentication, touch
  encoding, duplicate protection, password removal state, Unicode URI parsing,
  HOTP counters, nondefault periods, real QR image decoding and bounded inputs.
- Legacy settings fields and unknown fields survive round trips; malformed files
  remain intact; atomic saves work with Unicode paths, spaces and CRLF JSON.
- Real Linux desktop smoke exercised no-device state, Retry, navigation, About,
  add form, Ctrl+N, Cancel and Escape. Further preferences/form/resizing checks are
  being validated by `scripts/desktop-smoke.py`.
- Native clipboard test passed on isolated Xvfb: expiry clears an owned code,
  newer user text survives expiry, and shutdown clears the service's owned code.
- `cargo audit` has no vulnerability failures after updating Rustls and rqrr;
  four upstream warnings remain explicitly documented in SECURITY.md.
- Real screenshots are captured from the running Linux application, without a
  connected key or generated pretend credentials. No Windows/macOS screenshot
  has been fabricated.

## Required hardware QA — not performed

Use a disposable test key and record firmware, platform and build commit.

1. Insert/remove/reinsert while empty, locked, unlocked, busy, HOTP-only and with
   expired TOTP codes. Ensure removal clears identity/codes and retry recovers.
2. Add TOTP/HOTP credentials for supported SHA1/256/512 and 6/7/8 digits. Compare
   actual codes with independently computed RFC vectors using a public test
   secret. Verify 30/60-second periods and HOTP counter 42.
3. Set/change/remove an OATH password; reject wrong passwords and wrong mutual
   proofs. Reconnect and test automatic protected-key unlock prompting.
4. Test touch-required TOTP and HOTP, cancellation, timeout, unplug during touch,
   and retry. Cancellation discards the result; it cannot undo a hardware counter
   increment or interrupt every native transmit immediately.
5. Duplicate Add must not overwrite an existing secret. Failed Add must retain
   the form. Confirm Delete removes only the selected raw credential identifier.
6. Copy plain codes without spaces; reject expired codes; change clipboard
   contents before the timeout and during a blocked touch operation.
7. Verify Unicode filtering, empty search, row Enter/Shift+F10/context menu,
   service avatars, custom icon/reset and explicit opt-in favicon download/cache.

## Required platform/control QA

- Windows: actual MSI install/upgrade/uninstall, WebView2 absent/present, native
  file dialog, Unicode profile/paths, external links/folder opening, menu and
  Ctrl shortcuts, resizing/maximize/restore, high DPI, accessibility and key I/O.
- macOS Intel/Apple Silicon: open the actual .app bundle, Cmd shortcuts, app/Edit
  menus, native file dialog, Finder/browser integration, dark/system appearance,
  Retina, lifecycle, accessibility and key I/O; signing/notarization separately.
- Linux X11/Wayland: native/portal file chooser, QR file drop, theme/clipboard
  persistence, narrow/large windows, focus trap and all available controls.
- Flatpak: install the real bundle, run inside the sandbox, confirm host PC/SC,
  portal chooser/drop, clipboard ownership, themes, explicit favicon networking,
  icon/desktop/AppStream integration and restart/uninstall behavior.

Read CI logs and inspect every uploaded artifact. Verify SHA256SUMS and package
contents. Releases remain drafts until native and hardware gates are reviewed.
