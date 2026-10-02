# Building Gosh 2.0

The migration is a prerelease. [PLATFORM_SUPPORT.md](PLATFORM_SUPPORT.md) separates
tested behavior from pending native and hardware QA. Rust 1.98.0 is pinned in
`rust-toolchain.toml`; every build uses `rust/Cargo.lock`. Node, npm, a browser
server, GTK 4 and libadwaita are not required.

## Linux

On Ubuntu 24.04 or Debian with WebKitGTK 4.1:

```sh
sudo apt install build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev \
  libxdo-dev libpcsclite-dev pcscd libccid
sudo systemctl enable --now pcscd.socket
cargo build --locked --release --manifest-path rust/Cargo.toml
./rust/target/release/gosh-authenticator
```

Fedora equivalents are `gcc`, `pkgconf-pkg-config`, `gtk3-devel`,
`webkit2gtk4.1-devel`, `libxdo-devel`, `pcsc-lite-devel`, `pcsc-lite` and `ccid`.
Enable the host PC/SC service. GTK 3 is a Linux WebKitGTK/native-menu dependency;
the application UI is implemented in Dioxus on every OS.

Build a tarball with bundled metadata and license notices:

```sh
python3 scripts/package.py --platform linux --arch x64 \
  --binary rust/target/release/gosh-authenticator
./build-deb.sh                 # dpkg-deb required
./build-rpm.sh                 # rpmbuild required
```

Use `--arch arm64` for a binary actually compiled on ARM64. The tarball is
dynamically linked and requires the listed runtime libraries; it is not a static
binary or an AppImage. Flatpak provides the isolated Linux runtime.

## Windows

Use Windows 10/11 x64, the Visual Studio 2022 C++ desktop build tools and the MSVC
Rust toolchain. The Microsoft Edge WebView2 Evergreen Runtime must be installed;
download it from [Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/).
The repository config statically links the MSVC CRT; native CI still needs to
verify the result. Windows supplies WinSCard. Enable the Smart Card service and the key's CCID/OATH
interface. There is no pcsc-lite, GTK or WebKitGTK dependency on Windows.

```powershell
cargo build --locked --release --manifest-path rust/Cargo.toml
.\rust\target\release\gosh-authenticator.exe
dotnet tool install --global wix --version 6.0.2
py -m pip install Pillow==11.1.0
py scripts/package.py --platform windows --arch x64 --binary rust/target/release/gosh-authenticator.exe
```

The MSI installs the executable, first/third-party licenses, and a Start menu
shortcut. Windows Settings provides uninstall. Settings/cache remain in the
user profile after uninstall. The MSI does not install WebView2 automatically.
Prerelease MSI product versions use the numeric Cargo version; filenames keep
the full prerelease version. MSI files are unsigned until distribution signing
is configured. Windows ARM64 is not advertised as a supported artifact.

## macOS

Install Xcode Command Line Tools and Rust. macOS supplies WKWebView and the PCSC
framework. Build natively on Apple Silicon or Intel; no Homebrew GTK libraries
are needed. macOS 11 or later is the packaging minimum, pending native QA.

```sh
xcode-select --install
cargo build --locked --release --manifest-path rust/Cargo.toml
python3 -m pip install Pillow==11.1.0
python3 scripts/package.py --platform macos --arch arm64 \
  --binary rust/target/release/gosh-authenticator
```

Use `--arch x64` on Intel. The zip contains `Gosh Yubico Authenticator.app`,
Info.plist, icon, executable and licenses. The packager validates an ad-hoc code
signature. Developer ID signing and notarization are still needed for trusted
public distribution; no signed/notarized release is claimed. Drag the app to
Applications. Remove that bundle to uninstall; user preferences are retained.

## Flatpak

Install `flatpak` and `flatpak-builder`, then:

```sh
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user flathub org.gnome.Platform//50 org.gnome.Sdk//50
python3 flatpak/generate-cargo-sources.py --check
flatpak-builder --user --force-clean --install --repo=flatpak-repo \
  build-flatpak com.goshapps.YubicoAuthenticator.yml
flatpak run com.goshapps.YubicoAuthenticator
flatpak build-bundle flatpak-repo dist/gosh-authenticator.flatpak com.goshapps.YubicoAuthenticator
```

Run on x86_64 or aarch64; the manifest pins the matching Rust archive and SHA256
for each. The generated crate sources match Cargo.lock and compilation is
offline/locked. GNOME runtime supplies WebKitGTK; pcsc-lite supplies the client
and `--socket=pcsc` connects to the host daemon. File dialogs use the desktop
portal. No home filesystem or unrestricted USB permission is granted. Network
permission supports explicitly requested favicons, disabled by default.

The x86_64 SDK build, bundle export/install and no-device desktop controls were
exercised locally. The manifest builds libxdo explicitly because GNOME 50 does
not supply it. Sandbox selected-file/document portal QA remains blocked by the
cloud kernel's missing `/dev/fuse`; ARM64/native CI is blocked by account billing.
See QA.md for the precise scope and platform gates.

## Checks

```sh
cargo fmt --manifest-path rust/Cargo.toml --check
cargo clippy --locked --all-targets --all-features --manifest-path rust/Cargo.toml -- -D warnings
cargo test --locked --all-features --manifest-path rust/Cargo.toml
cargo test --locked --no-default-features --manifest-path rust/Cargo.toml
cargo audit --file rust/Cargo.lock
python3 -m pytest tests
python3 flatpak/generate-cargo-sources.py --check
appstreamcli validate --no-net data/metainfo/com.goshapps.YubicoAuthenticator.metainfo.xml
```

`scripts/run-linux-smoke.sh` drives the real desktop using AT-SPI on Xvfb. It
needs python3-pyatspi, python3-gi, python3-pil, xvfb, xauth, dbus-x11, xdotool and
AT-SPI. Hardware tests are explicitly ignored unless a disposable test YubiKey
is attached. Never run destructive credential/password QA on a production key.
See [QA.md](QA.md) for the manual gates.

Generate current notices with `scripts/generate-third-party-licenses.sh` after
installing `cargo-about --locked --features cli`. A Rustls advisory was fixed in
the lockfile; remaining upstream dependency warnings are listed in
[SECURITY.md](SECURITY.md).
