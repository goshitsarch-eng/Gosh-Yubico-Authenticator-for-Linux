# Platform support and verification

Version **2.0.0-alpha.1** is a migration prerelease. Implementation is not proof
of native runtime or hardware compatibility. The project is not ready for a
stable cross-platform release until the open QA gates below have evidence.

| Target | Implementation / artifact | Observed validation |
|---|---|---|
| Linux x86_64, X11 | Dioxus, WebKitGTK 4.1, pcsc-lite; tar.gz, DEB, RPM, Flatpak | Native build, software tests, real desktop navigation and native clipboard passed on Debian 13; expanded UI and packages in progress |
| Linux aarch64 | Same code; native CI and Flatpak jobs | Awaiting native CI and desktop/hardware QA |
| Linux Wayland | Wry window backend, portal dialog, arboard Wayland clipboard | Implemented; no Wayland compositor available for local QA |
| Flatpak x86_64 | GNOME 50, offline locked Cargo sources, PC/SC socket, portals | Actual SDK build in progress; sandbox install/runtime QA pending |
| Flatpak aarch64 | Pinned ARM64 Rust archive and separate CI job | Awaiting CI build and sandbox QA |
| Windows 10/11 x64 | WebView2, WinSCard, native dialogs/clipboard/menu, WiX MSI | Awaiting native CI compile, MSI install/uninstall and interaction/hardware QA |
| macOS Apple Silicon | WKWebView, PCSC framework, native menus/dialogs/clipboard, .app zip | Awaiting native CI compile/bundle and interaction/hardware QA |
| macOS Intel | Same architecture with native Intel build job | Awaiting native CI compile/bundle and interaction/hardware QA |
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
the intended release baseline, pending actual workflow results.

See [QA.md](QA.md), [MIGRATION_AUDIT.md](MIGRATION_AUDIT.md) and
[BUILDING.md](BUILDING.md). Record OS, architecture, build commit and actual
results when closing a gate. A green build does not close manual UI/hardware QA.
