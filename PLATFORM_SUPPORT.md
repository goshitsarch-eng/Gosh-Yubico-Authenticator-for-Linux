# Platform support and verification

Version **2.0.0-alpha.1** is a migration prerelease. Implementation is not proof
of native runtime or hardware compatibility. The project is not ready for a
stable cross-platform release until the open QA gates below have evidence.

| Target | Implementation / artifact | Observed validation |
|---|---|---|
| Linux x86_64, X11 | Dioxus, WebKitGTK 4.1, pcsc-lite; tar.gz, DEB, RPM, Flatpak | Native build, software tests, real desktop navigation and native clipboard passed on Debian 13; all available no-device preferences/form controls, native portal QR import, Ctrl+Q and unpacked DEB UI passed; tar/DEB/RPM built and inspected |
| Linux aarch64 | Same code; native CI and Flatpak jobs | CI requested but never started (account billing); desktop/hardware QA unrun |
| Linux Wayland | Wry window backend, portal dialog, arboard Wayland clipboard | Implemented; no Wayland compositor available for local QA |
| Flatpak x86_64 | GNOME 50, offline locked Cargo sources, PC/SC socket, portals | Actual offline SDK build, bundle export/install, CLI and real sandbox UI controls/settings passed; document portal blocked by missing /dev/fuse |
| Flatpak aarch64 | Pinned ARM64 Rust archive and separate CI job | CI requested but never started (account billing); sandbox QA unrun |
| Windows 10/11 x64 | WebView2, WinSCard, native dialogs/clipboard/menu, WiX MSI | CI requested but never started (account billing); MSI/native interaction and hardware QA unrun |
| macOS Apple Silicon | WKWebView, PCSC framework, native menus/dialogs/clipboard, .app zip | CI requested but never started (account billing); bundle/native interaction and hardware QA unrun |
| macOS Intel | Same architecture with native Intel build job | CI requested but never started (account billing); bundle/native interaction and hardware QA unrun |
| Windows ARM64 | No advertised artifact | Unsupported in this prerelease; no native runner or QA evidence |

On Linux, enable `pcscd` and CCID support. Windows uses the Smart Card service;
macOS provides the PCSC framework. Enable the YubiKey's CCID/OATH interface.
No physical key is attached to the cloud environment. Protocol fixtures validate
encoding, parsing, mutual-authentication proofs and counter/period preservation;
they do not validate USB access, real touch timing, firmware capability, password
changes or interoperability with an actual YubiKey.

Windows installers require Microsoft's WebView2 Evergreen Runtime. macOS bundles
are ad-hoc signed, not Developer ID signed or notarized. No trusted public
distribution is claimed. Linux tarballs use dynamic system libraries; artifacts
compiled locally on Debian 13 require its ABI. CI builds on Ubuntu 24.04 provide
the intended release baseline, not verified because the workflow jobs never started.

See [QA.md](QA.md), [MIGRATION_AUDIT.md](MIGRATION_AUDIT.md) and
[BUILDING.md](BUILDING.md). Record OS, architecture, build commit and actual
results when closing a gate. A green build does not close manual UI/hardware QA.

## Feature evidence

`Implemented` means code exists; `Observed` means the named runtime check ran.
Protocol fixtures are software evidence, never a substitute for a physical key.

| Feature | Windows / macOS | Linux X11 | Flatpak x86_64 |
|---|---|---|---|
| Shared Dioxus UI, navigation, About/license | Implemented; CI blocked | Observed with AT-SPI and real screenshots | Observed by real input and screenshots; WebView AT-SPI traversal unavailable in this session |
| Light / Dark / System | Implemented; native appearance QA unrun | Observed live, including native theme integration | Live selectors/persistence observed; final screenshots recorded separately |
| Legacy JSON/settings/window state | Portable software tests; native paths unrun | Roundtrip/corruption/atomic/Unicode/CRLF tests and real preference persistence observed | Sandbox preference persistence and retained unknown field observed |
| QR/URI, periods/counters | Implemented; native dialog/drop unrun | Real image tests, Unicode paste and native portal file selection observed; drop unrun | URI paste observed; selected-file portal blocked by /dev/fuse; drop unrun |
| Native menus/shortcuts/lifecycle | Implemented; native interaction unrun | Ctrl+N, Escape, Ctrl+I, Ctrl+F focus/paste from another page and Ctrl+Q observed; other OS shortcuts/manual gates remain | Ctrl+N/Escape and native menus observed |
| Clipboard | Implemented; native test unrun | Real native expiry/replacement/shutdown test passed | Pasted Unicode text observed; code copying/expiry requires real key QA |
| Connect/auth/add/delete/calculate/touch | Implemented; physical key unrun | No-device UI and protocol/state regression tests passed; actual key unrun | Real no-device UI observed; actual key unrun |
| Icons/favicons, browser/file manager | Implemented; native/key flows unrun | Domain/cache/image validation tested; device icon dialogs and registered external apps need manual QA | Implemented with portal/opt-in network; external/document/key flows unrun |

All hardware/platform/manual gates remain listed in QA.md. The measured Linux
debug startup and process-tree memory regression is open; this prerelease does
not meet the requested performance or complete cross-platform release gates.
