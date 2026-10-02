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
count as verification of those environments. GitHub API access returned Forbidden during the baseline audit and became
available on 2026-10-02. Remote workflow results must be recorded separately
from local checks; do not mark CI or releases passed prematurely.

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
`UNKNOWN` means hardware or platform evidence is unavailable. The initial inventory is preserved in commit `0d092d0`. `MIGRATED` below
means implemented; it does not close a hardware or native-platform QA gate.

| Feature | Current state / source | Expected behavior and defects | Migration status / platform considerations |
|---|---|---|---|
| Credentials / Key Info / Settings navigation | WORKING, `ui/window.rs` | Three tabs, responsive header/bottom switcher | VERIFIED Linux X11: navigation/accessibility smoke; shared Dioxus components |
| Connect / Retry / disconnect | PARTIAL, worker + connection | First reader with OATH; no hotplug retry; errors can leave stale session | MIGRATED `services/runtime.rs`, `connection.rs`: retry, presence polling, explicit disconnect; no-device UI verified, hotplug hardware gate |
| Password unlock | UNKNOWN, `oath.rs`, dialogs | Password-derived key currently 20 bytes, Yubico uses 16; advertised HMAC algorithm ignored; ordinary equality | MIGRATED `oath.rs`: 16-byte key, advertised HMAC, constant-time proof; protocol regression verified; hardware gate |
| Set/change/remove password | UNKNOWN, `oath.rs` | Minimum four characters in UI; password-protection state stays stale | MIGRATED `oath.rs`, dialogs: protection state updates, confirmation; protocol regression verified; hardware gate |
| Require PIN on launch | DEAD, `prefs.rs`, Settings | Persisted/displayed but never read during connection; cannot secure an unprotected key | MIGRATED runtime/UI protected-key prompt; preferences persistence verified; locked-key hardware gate |
| List credentials and issuer/account | UNKNOWN, `oath.rs` | Unsupported type/algorithm silently default; malformed TLV tails ignored | MIGRATED strict list/SELECT/TLV parsing; malformed-response tests verified; real key gate |
| TOTP calculation + countdown | UNKNOWN, `oath.rs`, window timer | Every 30 seconds; custom periods ignored | MIGRATED credential periods/expiry in runtime; custom-period protocol tests verified; hardware countdown gate |
| HOTP calculation | PARTIAL, worker / row | Counter imported then discarded; HOTP tagged as touch by calculate-all | MIGRATED explicit HOTP Generate; counter 42 regression verified; real counter/touch gate |
| Touch-required credentials | PARTIAL, worker/dialog | Touch status conflated with HOTP; modal arrives after attempted calculation, cancellation does not cancel PC/SC | MIGRATED background touch prompt and generation cancellation; cannot interrupt every native transmit or undo HOTP; hardware gate |
| Search/filter | WORKING software, `ui/state.rs` | ASCII lowercase only; empty filtered list misleadingly says no credentials | MIGRATED Unicode state filtering/distinct empty state; reducer tests verified; populated UI hardware gate |
| Add issuer/account/secret/type/algorithm/digits/touch | PARTIAL, `add_page.rs` | Submission closes and clears before success; invalid/empty secret validation inconsistent; disconnected form can submit | MIGRATED validated form retained until success; all available fields and disabled disconnected Save verified in Linux UI |
| HOTP initial counter | BROKEN import, `add_page.rs` | Always writes zero; no editable counter | VERIFIED parser/form/protocol counter 42; actual YubiKey counter gate |
| Nondefault TOTP periods | BROKEN import, `qr.rs` / form | Parser accepts then form discards; period prefix not interpreted in names | VERIFIED URI/form/protocol 60-second period and UTF-8 name; actual code timing gate |
| QR file import | PARTIAL, `qr.rs` | PNG/JPEG/WebP; only first QR, blocking/unbounded decode | MIGRATED bounded worker decode/native portal picker/file drop; actual QR image tests verified; native portal selection/import verified; sandbox portal blocked by /dev/fuse; drop QA pending |
| Pasted otpauth URI | PARTIAL, `qr.rs` | UTF-8 corruption, duplicate/invalid values accepted, issuer mismatch ignored | VERIFIED strict parser regression and actual Unicode URI paste into Linux UI |
| Copy code / transient copied feedback | PARTIAL, row + clipboard | Copies formatted code with spaces; expiry overwrites unrelated newer clipboard contents | MIGRATED plain codes/expiry validation; native clipboard ownership, expiry and shutdown tests verified; device Copy gate |
| Clipboard expiry preferences | WORKING persistence, Settings | Five durations; filesystem failures swallowed | VERIFIED all five selectors and atomic persistence in Linux UI; legacy unknown fields retained |
| Credential context menu | UNKNOWN device flow, window | Copy, Calculate, Choose icon, Delete; mouse-only | MIGRATED shared commands, row Enter/Shift+F10/context actions; populated device UI gate |
| Delete with irreversible warning | UNKNOWN, dialogs | Confirm then worker deletes raw name | MIGRATED explicit irreversible confirmation and exact raw identifier; hardware gate |
| Service color avatars / custom icon / Reset | WORKING mapping tests, icons/dialogs | 31 built-in labels/colors; selection has little feedback | MIGRATED 31 service keys/colors, accessible selection/reset; mapping tests verified; device icon dialog gate |
| Favicon customization + cache | PARTIAL/Flatpak BROKEN, window | Privacy/path/bounds/retry defects above | MIGRATED opt-in HTTPS, validated domains, bounded PNG normalization and hashed cache with legacy fallback; device/cache/network UI gate |
| Device firmware and counts | UNKNOWN, Key Info | Identifies model as firmware major (not actual product identity) | MIGRATED OATH firmware and counts; no fabricated product model; actual device gate |
| Follow System / Light / Dark | WORKING, `prefs.rs` | Dynamic GTK theme; explicit choice persisted | VERIFIED Linux live Light/Dark/System selectors and settings persistence; OS/Wayland appearance gates |
| About, website, issue URL, license | WORKING source/menu, dialogs | Hardcoded version; inherited GTK About behavior | VERIFIED Linux About content/Cargo version; native menus and HTTPS links implemented; browser/platform gates |
| Quit / window lifecycle | WORKING, menu | Window size/position not persisted | MIGRATED window state, native close/quit and clipboard shutdown; resize/clipboard tests verified; lifecycle/platform gates |
| Keyboard shortcuts / drag and drop | NOT IMPLEMENTED baseline | Add shortcuts and QR image drop where supported | MIGRATED Ctrl/Cmd shortcuts and native QR drop; Linux Ctrl+N/Escape, Ctrl+I, Ctrl+F focus/paste and Ctrl+Q verified; remaining OS/drop gates |
| Windows / macOS packages | NOT IMPLEMENTED | No installer/bundle or QA | MIGRATED WiX MSI and signed app-bundle scripts plus native CI jobs; MSI/app bundle CI requested but blocked before any step by account billing; native install/QA pending |
| Flatpak / Linux release packages | PARTIAL, manifests/scripts | Existing x86_64 Rust archive only; README incorrectly describes SDK extension; RPM/DEB omit license material | MIGRATED native tar/DEB/RPM, both Flatpak architectures/offline lock sources, notices; local tar/DEB/RPM and actual x86_64 Flatpak build/export/install/UI verified; ARM64/release/manual gates pending |

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
- [x] New architecture documented and implemented (ARCHITECTURE.md).
- [ ] Each inventory row has a concrete implementation and regression evidence.
- [x] New Linux UI launched; all available no-device preferences/form controls, About/license, portal import and quit exercised. External browser/file-manager and device controls remain explicit manual gates.
- [ ] Windows and macOS CI builds and native QA recorded separately.
- [x] Actual x86_64 Flatpak built, installed and exercised inside its sandbox; selected-file portal and ARM64 gates remain open.
- [ ] Release artifacts and checksums exist and release workflow has run.
- [x] Documentation, current notices and the feature comparison reflect observed results and open gates.

Do not check external platform/hardware boxes on the basis of mocks or source
inspection. Record each blocked operation and its precise prerequisite.

## Evidence and deliberate differences (2026-10-02)

The Dioxus frontend is now canonical. Uncompiled GTK widgets, old preferences,
clipboard/worker duplicates and the placeholder C CodeQL file were removed after
software feature comparison and real Linux UI checks. The original code remains
in Git and the local reference snapshot. Historical binary distributions and the
old screenshot were removed from the source checkout; generated artifacts belong
in CI/release storage. This cleanup does not claim physical-device parity.

Touch cancellation discards late results; PC/SC calls and hardware HOTP counter
increments cannot always be cancelled. Favicons require explicit consent instead
of disclosing inferred account domains automatically. Duplicate Add refuses
overwrite instead of replacing secrets silently. A custom legacy cache file is
read only after hostname/image validation; new cache filenames use SHA-256 to
avoid Windows reserved names and case collisions. Model labels now report OATH
firmware rather than guessing a YubiKey model from its major version. No useful
workflow has been intentionally removed.

Software evidence: library/protocol/real QR-image tests, settings compatibility
and corruption/atomic-write tests, isolated native clipboard expiry/replacement/
shutdown test, and accessibility-driven Linux UI checks. The latter exercises
all available no-device navigation, all preferences, About, add-form fields,
Unicode URI paste, HOTP counter/digits/algorithm, Ctrl+N/Escape and 420-pixel
resizing. Generated QR and protocol fixtures exist only in tests. Production
connects to real PC/SC and displays no fabricated credential data.

Hardware-dependent dialogs/actions, Windows/macOS full interaction, Wayland,
high DPI/Retina, portal/drop integration and complete sandbox QA remain open
until their evidence is recorded in QA.md and PLATFORM_SUPPORT.md.

Final review retained About's full first-party license view and added access to
compiled third-party notices through native file opening. Explicit appearance
now also updates native window/menu theme. Authentication regression coverage
includes advertised SHA256/SHA512 mutual proofs and rejects malformed/duplicate
SELECT authentication metadata; SET_CODE deliberately establishes SHA1. HOTP
operations offer cancellation and a conditional touch instruction because the
key's list does not report their touch policy.

An actual Ctrl+F test from Key Info exposed a focus race before the credentials
page mounted. Search now waits for mounting and verifies the active DOM element,
working around Desktop 0.7.3's invalid focus-result type. Real Unicode paste and
absence of the false error banner are checked. Cached icon reads
also enforce their bound on the open file handle, avoiding a metadata/read race.
Restart QA exposed dynamically inserted select options displaying the first
entry rather than the persisted value. Explicit option selection fixes both
preferences and credential fields; visible selected values and a fresh-process
preferences test cover this regression.

Measured debug readiness increased from 0.313 s to 0.518 s and process-tree PSS
from 92.7 MiB to 331.9 MiB. Performance parity is **not achieved**. CI billing,
physical hardware, native platform interaction, Wayland/high DPI and complete
portal QA still prevent a declaration that the migration is complete.
