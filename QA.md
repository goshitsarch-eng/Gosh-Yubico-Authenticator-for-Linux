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
  Before the review follow-up, all-features and headless runs each passed 39 software tests; the
  separately executed native clipboard test brings the total to 40 unique Rust
  tests. The physical connection test remains ignored. Thirteen Python metadata
  and release/package validation tests pass. Rustfmt and strict Clippy pass.
- The review follow-up adds four regression tests. The current desktop run
  passes 43 software tests, and the headless run passes 41; Rustfmt and strict
  desktop/headless Clippy pass. Device dialogs close on removal or a changed
  device identity, with a fresh component for an immediately reopened unlock
  prompt. Serialized field-level preference edits preserve unrelated settings,
  interleaved window saves, icon edits and unknown fields while acknowledgements
  remain unread. Invalid edits preserve both worker state and the saved file.
  The rebuilt native desktop also passed the real controls smoke, including
  all preference selectors/toggles, legacy persistence, navigation, About,
  credential fields, shortcuts, narrow resizing and clean Quit. Physical key
  swaps still require hardware QA.
- Legacy settings fields and unknown fields survive round trips; malformed files
  remain intact; atomic saves work with Unicode paths, spaces and CRLF JSON.
- Real Linux desktop smoke exercised no-device state, Retry, navigation, About,
  every theme/clipboard selector, both preference toggles, About/license details,
  Unicode URI paste, HOTP counter/digits/algorithm/touch fields, disabled no-device
  Save, Ctrl+N, Cancel/Escape and 420-pixel resizing. A native portal chooser
  imported an actual public-test QR image from a filename with spaces and é;
  Ctrl+I, Ctrl+F from Key Info (including actual Unicode paste into the focused
  search field) and Ctrl+Q exit passed. Unknown legacy fields survived saves.
- A fresh application process restored the visibly selected Dark/120-second
  preferences and both toggles after real UI changes and saving. This exposed
  and fixed an initial dropdown-selection defect; options now reflect persisted
  values at mount. The Linux CI smoke includes the restart regression check.
  That complete CI smoke wrapper ran locally against the final extracted RPM;
  this is local evidence, separate from the unstarted GitHub jobs below.
- Native clipboard test passed on isolated Xvfb: expiry clears an owned code,
  newer Unicode user text survives expiry, and shutdown clears the service's
  owned code.
- `cargo audit` has no vulnerability failures after updating Rustls and rqrr;
  four upstream warnings remain explicitly documented in SECURITY.md.
- Real screenshots are captured from the running Linux application, without a
  connected key or generated pretend credentials. No Windows/macOS screenshot
  has been fabricated.

## Artifact and workflow evidence

The package and performance evidence below was collected at migration commit
`39ebe3d`, before the dialog and preference review fixes. These artifacts were
not rebuilt for that follow-up.

- Optimized Linux tar.gz, DEB and RPM were produced, extracted and inspected for
  executable, icon/desktop/AppStream metadata and first/third-party licenses.
  Their actual executables reported 2.0.0-alpha.1. The extracted DEB passed the
  real desktop controls smoke; the extracted RPM also passed the controls,
  native QR import and Quit check. The DEB and tar contain identical executables.
  These local artifacts use Debian 13's ABI, not the unrun Ubuntu CI baseline.
- The actual GNOME 50 x86_64 Flatpak compiled offline/locked, exported, bundled
  and installed. Its sandboxed UI accepted actual mouse/keyboard navigation,
  all theme/clipboard selectors, protected-key prompt preference, Unicode URI
  paste, Ctrl+N/Escape and 420-pixel resizing. Real screenshots were inspected.
  Persistent sandbox JSON retained an unknown legacy field. No home/all-USB
  application permission was added. The host private PC/SC socket was exposed
  at its normal host location for the existing `--socket=pcsc` permission.
  The final installed stable-branch commit is
  `13d8836fac1daf99820ad16caebd2244736bc48854ce799b612457675ef15fb1`.
  Its CLI reports 2.0.0-alpha.1; [preferences](screenshots/flatpak-settings-dark.png)
  and [Unicode form](screenshots/flatpak-add-credential.png) captures come from
  this actual sandbox session.
- The same AT-SPI traversal used for the native WebView cannot access the
  sandboxed WebView in this cloud session. Its narrower mouse/keyboard checks
  are recorded separately; the failing accessibility traversal was not labelled
  passed. The document portal reports missing `/dev/fuse`, so selected sandbox
  files, file drop, external document opening and complete portal QA remain open.
- Native/Flatpak CI was requested in [run 36947916875](https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux/actions/runs/36947916875)
  and the preceding push run. Every job has zero executed steps. GitHub's check
  annotation states: “The job was not started because your account is locked
  due to a billing issue.” No Windows/macOS/ARM64 artifacts were produced,
  downloaded or verified. Resolve the owner account billing lock, then rerun CI
  and inspect actual artifacts before claiming those targets/release automation.
- Release collection guard tests reject an incorrect tag/incomplete artifacts
  before creating output. Packaging validates actual ELF/PE/Mach-O CPU headers
  to prevent architecture mislabelling. These tests are not native installer QA.
  No release tag or published release was created.

Local `dist/SHA256SUMS` checks all four actual x86_64 artifacts (tar.gz, DEB,
RPM, Flatpak). Build outputs are ignored by Git; they are not a complete native
release set or remotely uploaded CI artifacts. The Flatpak build helper was
also executed successfully with the Cargo prerelease version and pinned Rust
archive, without installing an unused SDK Rust extension.

## Performance gate — open

Three warm-cache debug samples on Debian 13/Xvfb, with the same real PC/SC
no-reader service, were measured using `scripts/measure-linux.py`. Readiness is
when an AT-SPI client observes the no-device label; PSS sums the app process tree
two seconds later. Both builds are debug builds; these are development results,
not release, GPU, high-DPI or populated-key performance claims.
UI revision `39ebe3d` was measured again; [raw samples and binary/source
digests](evidence/linux-debug-performance-2026-10-02.json) record this run.

| Build | Median observed readiness | Median process-tree PSS | Processes |
|---|---|---|---|
| GTK baseline 1.2.1 | 0.313 s | 92.7 MiB | 1 |
| Dioxus development build | 0.518 s | 331.9 MiB | 3 |

The WebView migration has a measured startup/memory regression. Profiling and
optimization, optimized baseline comparisons, scrolling/filtering with real
credential lists and Windows/macOS measurements remain required. This gate is
not closed by a successful build or fast no-device interaction.

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
