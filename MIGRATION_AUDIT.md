# Migration audit

Baseline: commit `8cc4be3537c6f054501a68b03784aae074a7ecff`, version 1.2.1.
Audit began 2026-10-01. Migration target: **2.0.0-alpha.1**, Rust + Dioxus Desktop.
This is a prerelease migration, not a claim of finished cross-platform QA.

## Evidence and verification boundaries

The original binary was built and launched on Debian 13 x86_64 with GTK 4.18.6,
Adwaita 1.7.6 and PC/SC 2.3.3. The original native build, 20 Rust tests, five
packaging tests, Flatpak crate-source consistency and AppStream validation passed.
The existing hardware test was ignored by its runner. An accessibility-driven
desktop session exercised Credentials, Retry, Key Info and Settings and observed
the real no-device state. Further menu, preferences and add-dialog inspection is
recorded in the local audit logs. A reference binary/source snapshot is outside
the checkout at `/workspace/.cloud-environment/gosh/reference`; Git retains the
complete baseline. No existing source was deleted before this audit.

No physical YubiKey is attached. Hardware-dependent behavior below is **source
audited**, not hardware verified. Windows, macOS, Wayland and Flatpak runtime QA
must have independent evidence; neither Linux compilation nor simulated devices
count as verification of those environments. GitHub API access currently returns
Forbidden in this cloud environment; remote workflow/release execution requires
usable GitHub API/network access. Do not mark CI or releases passed prematurely.

## Current architecture

`rust/src/main.rs` initializes logging, a GTK/Adwaita application, CSS and icon
paths. `ui/window.rs` (over 1,100 lines) builds the three-tab window and owns most
rendering, interaction, context-menu, countdown and favicon behavior. Components
are linked by `Rc`, `RefCell`, `Cell`, and deeply nested callbacks. `ui/state.rs`
mixes runtime device state, preferences, clipboard state, search, timers and UI
dialog flags. A dedicated blocking worker in `services/yubikey_service.rs` owns
an OATH session and uses bounded async-channel queues; the UI polls every 80 ms.
Commands use blocking sends on the UI thread. Error handling frequently discards
filesystem errors or converts invalid data into defaults.

The reusable library exports `core` and `services`. Core models/APDU/OATH parsing
are Rust, PC/SC binds the OS smart-card stack. GTK 4 + libadwaita is the primary
UI. Other major crates: hmac, SHA-1/SHA-2, PBKDF2, getrandom, data-encoding,
image (all default codecs), rqrr, url, serde/JSON, dirs, ureq and async-channel.
`base64`, `sha2`, and several utility functions are unused or incompletely used.

### Source and entry points

| Location | Responsibility |
|---|---|
| `rust/src/main.rs`, `lib.rs` | Desktop entry point; reusable core exports |
| `core/credential.rs` | Credential identifiers, code formatting, Base32 |
| `core/yubikey/{apdu,connection,oath,error}.rs` | TLV/APDU, PC/SC, password and OTP operations |
| `services/yubikey_service.rs` | Worker commands/events |
| `ui/{window,state,dialogs,add_page}.rs` | Three tabs, modals, credential form |
| `prefs.rs`, `clipboard.rs`, `qr.rs`, `service_icons.rs` | Settings, clipboard, QR/URI, avatar mapping |
| `build-*.sh`, `.github/workflows`, Flatpak manifest | Distribution/release |
| `tests/test_packaging.py` | Packaging/license/trademark assertions |

There are no application shell commands (`std::process::Command`), document
save/export workflows, undo/redo, printing, file associations, or drag/drop in
the baseline. GTK/GApplication provides inherited command-line behavior; there
is no app-specific command-line interface. No application keyboard accelerators
are registered. Native GTK focus navigation is available; credential activation
is a mouse gesture rather than a keyboard-accessible action.

## Data, persistence, networking and platform assumptions

Credentials and secret keys live on the YubiKey OATH applet. The app reads and
mutates them through PC/SC; there is no local credential database or secret
export. IDs are the raw UTF-8 credential-name bytes. Settings are JSON at
`dirs::config_dir()/gosh-authenticator/settings.json`: `theme_mode` (lowercase
system/light/dark), `clipboard_timeout_seconds` (10/20/30/60/120),
`require_pin_on_launch`, `icon_prefs` keyed by standard padded Base64 of the
raw credential ID, and `favicon_cache` (declared but unused). Each icon preference
has optional `custom_icon_key` and `favicon_domain`. These keys/values must remain
readable; new optional fields must not invalidate old files. Bad JSON currently
silently resets the UI to defaults; saves silently fail and are non-atomic.

Favicons cache under `dirs::cache_dir()/gosh-authenticator/favicons/DOMAIN.png`.
The app sends inferred account domains to Google's favicon endpoint without
explicit consent. Domain input is not safely constrained before it becomes a
filename, responses are read without a size bound, concurrent requests are not
deduplicated, and completion does not notify the UI. The Flatpak lacks network
permission, making this behavior ineffective inside its sandbox. The rewrite
must preserve optional icon customization while preventing implicit disclosure
of account domains and path traversal.

QR import reads PNG/JPEG/WebP and an `otpauth://` URI. The form additionally
recognizes URIs pasted into its secret field. No local secrets are written.
Native QR decoding currently runs in the UI callback without image-size limits.
The parser hand-decodes UTF-8 incorrectly and silently defaults malformed numbers.

Linux assumptions: GTK 4.14, Adwaita 1.5, `/app` and `/usr/share` icon paths,
host pcscd/systemd instructions, Linux RPM/DEB scripts, bash-based packaging and
GNOME Flatpak runtime. Runtime core should retain OS-native PC/SC (WinSCard on
Windows, PCSC framework on macOS, pcsc-lite on Linux); application runtime must
not shell out to Linux utilities. Desktop dialogs/clipboard/browser integration
must use cross-platform APIs and Flatpak portals.

## Feature inventory

`WORKING` means observed or existing tests support software behavior;
`UNKNOWN` means hardware or platform evidence is unavailable. Migration status
starts `NOT IMPLEMENTED` and must be updated only with concrete evidence.

| Feature | Current state / source | Expected behavior and defects | Migration status / platform considerations |
|---|---|---|---|
| Credentials / Key Info / Settings navigation | WORKING, `ui/window.rs` | Three tabs, responsive header/bottom switcher | NOT IMPLEMENTED; shared Dioxus components |
| Connect / Retry / disconnect | PARTIAL, worker + connection | First reader with OATH; no hotplug retry; errors can leave stale session | NOT IMPLEMENTED; OS-native PC/SC on all platforms |
| Password unlock | UNKNOWN, `oath.rs`, dialogs | Password-derived key currently 20 bytes, Yubico uses 16; advertised HMAC algorithm ignored; ordinary equality | NOT IMPLEMENTED; protocol tests + hardware gate |
| Set/change/remove password | UNKNOWN, `oath.rs` | Minimum four characters in UI; password-protection state stays stale | NOT IMPLEMENTED; preserve explicit removal confirmation |
| Require PIN on launch | DEAD, `prefs.rs`, Settings | Persisted/displayed but never read during connection; cannot secure an unprotected key | NOT IMPLEMENTED; implement actual protected-key unlock policy |
| List credentials and issuer/account | UNKNOWN, `oath.rs` | Unsupported type/algorithm silently default; malformed TLV tails ignored | NOT IMPLEMENTED; fail closed |
| TOTP calculation + countdown | UNKNOWN, `oath.rs`, window timer | Every 30 seconds; custom periods ignored | NOT IMPLEMENTED; per-credential period and expiry |
| HOTP calculation | PARTIAL, worker / row | Counter imported then discarded; HOTP tagged as touch by calculate-all | NOT IMPLEMENTED; explicit Generate advances counter, Copy never does |
| Touch-required credentials | PARTIAL, worker/dialog | Touch status conflated with HOTP; modal arrives after attempted calculation, cancellation does not cancel PC/SC | NOT IMPLEMENTED; prompt before background operation, explicit cancellation |
| Search/filter | WORKING software, `ui/state.rs` | ASCII lowercase only; empty filtered list misleadingly says no credentials | NOT IMPLEMENTED; Unicode-aware search and distinct empty results |
| Add issuer/account/secret/type/algorithm/digits/touch | PARTIAL, `add_page.rs` | Submission closes and clears before success; invalid/empty secret validation inconsistent; disconnected form can submit | NOT IMPLEMENTED; preserve form on failure, validate at domain boundary |
| HOTP initial counter | BROKEN import, `add_page.rs` | Always writes zero; no editable counter | NOT IMPLEMENTED; preserve URI counter and expose input |
| Nondefault TOTP periods | BROKEN import, `qr.rs` / form | Parser accepts then form discards; period prefix not interpreted in names | NOT IMPLEMENTED; preserve period per YKOATH naming |
| QR file import | PARTIAL, `qr.rs` | PNG/JPEG/WebP; only first QR, blocking/unbounded decode | NOT IMPLEMENTED; bounded background decode, native/portal file dialog |
| Pasted otpauth URI | PARTIAL, `qr.rs` | UTF-8 corruption, duplicate/invalid values accepted, issuer mismatch ignored | NOT IMPLEMENTED; strict parser and regression tests |
| Copy code / transient copied feedback | PARTIAL, row + clipboard | Copies formatted code with spaces; expiry overwrites unrelated newer clipboard contents | NOT IMPLEMENTED; plain code, compare actual clipboard before clearing |
| Clipboard expiry preferences | WORKING persistence, Settings | Five durations; filesystem failures swallowed | NOT IMPLEMENTED; worker + validated atomic settings |
| Credential context menu | UNKNOWN device flow, window | Copy, Calculate, Choose icon, Delete; mouse-only | NOT IMPLEMENTED; context + keyboard entry points share commands |
| Delete with irreversible warning | UNKNOWN, dialogs | Confirm then worker deletes raw name | NOT IMPLEMENTED; require connected/unlocked device and explicit confirm |
| Service color avatars / custom icon / Reset | WORKING mapping tests, icons/dialogs | 31 built-in labels/colors; selection has little feedback | NOT IMPLEMENTED; retain keys and accessible selected state |
| Favicon customization + cache | PARTIAL/Flatpak BROKEN, window | Privacy/path/bounds/retry defects above | NOT IMPLEMENTED; opt-in only, validated domains, bounded cached images |
| Device firmware and counts | UNKNOWN, Key Info | Identifies model as firmware major (not actual product identity) | NOT IMPLEMENTED; show OATH firmware honestly |
| Follow System / Light / Dark | WORKING, `prefs.rs` | Dynamic GTK theme; explicit choice persisted | NOT IMPLEMENTED; system media query + live explicit theme |
| About, website, issue URL, license | WORKING source/menu, dialogs | Hardcoded version; inherited GTK About behavior | NOT IMPLEMENTED; derive Cargo version, native app menus on macOS |
| Quit / window lifecycle | WORKING, menu | Window size/position not persisted | NOT IMPLEMENTED; window state and conventional accelerators |
| Keyboard shortcuts / drag and drop | NOT IMPLEMENTED baseline | Add shortcuts and QR image drop where supported | NOT IMPLEMENTED; Ctrl/Cmd platform convention |
| Windows / macOS packages | NOT IMPLEMENTED | No installer/bundle or QA | NOT IMPLEMENTED; MSI, app bundle, platform CI |
| Flatpak / Linux release packages | PARTIAL, manifests/scripts | Existing x86_64 Rust archive only; README incorrectly describes SDK extension; RPM/DEB omit license material | NOT IMPLEMENTED; offline locked sources, both architectures, sandbox QA |

## Existing tests and important bugs

14 library tests: 13 passed, one hardware connection ignored. Seven binary tests:
preferences (2), URI parsing (3), service icons (2). Five Python packaging tests
check license installation, pinned Flatpak Rust, generated notices, trademark
notice and notice generator. There are no device mocks, state transition,
clipboard ownership, settings error/migration, protocol authentication, QR image,
window lifecycle or automated application interaction regression tests.

Priority fixes: YKOATH authentication derivation/algorithm; malformed response
handling; settings durability and compatibility; clipboard ownership; URI UTF-8,
counter/period and validation; asynchronous QR import; stale session/code state;
touch cancellation and prompts; duplicate credential replacement; unconsented
favicon networking/path traversal; misleading empty-search state; dead PIN
preference; keyboard accessibility. The existing UI is visually coherent, so
retain its branding, blue accent, three primary sections and hardware-first
security model while removing its toolkit-specific callback architecture.

## Migration ledger

- [x] Baseline built and launched; original source and data formats audited.
- [x] Feature inventory and known limitations recorded before rewriting.
- [ ] New architecture documented and implemented.
- [ ] Each inventory row has a concrete implementation and regression evidence.
- [ ] New Linux UI launched and every available control exercised.
- [ ] Windows and macOS CI builds and native QA recorded separately.
- [ ] Actual Flatpak built, installed and exercised inside its sandbox.
- [ ] Release artifacts and checksums exist and release workflow has run.
- [ ] Documentation, license notices and final feature comparison agree.

Do not check external platform/hardware boxes on the basis of mocks or source
inspection. Record each blocked operation and its precise prerequisite.
