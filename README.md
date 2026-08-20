# Gosh Yubico Authenticator for Linux

A native GTK 4 / Adwaita desktop application for managing OATH (TOTP/HOTP)
credentials on YubiKey devices.

## Features

Manage OATH credentials with TOTP/HOTP code generation, password-protected
YubiKeys, touch-required credentials, clipboard auto-clear, QR import, and
system / light / dark theme support.

## Screenshots

![Gosh YubiKey Manager – Credentials View](screenshots/img1.png)

## Quick Start

**Prerequisites:** Install `pcscd` (PC/SC Smart Card Daemon) for YubiKey
communication.

```bash
# Fedora/RHEL
sudo dnf install pcsc-lite

# Ubuntu/Debian
sudo apt install pcscd

# Arch Linux
sudo pacman -S pcsclite
```

**Enable and start the daemon:**

```bash
sudo systemctl enable --now pcscd
```

**Run the application:**

```bash
# From a release tarball:
./gosh-authenticator

# Or install the package:
sudo dnf install gosh-authenticator-*.rpm
sudo apt install ./gosh-authenticator_*.deb
```

## Building from Source

**Build dependencies:** Rust 1.70+, GTK 4, libadwaita, libpcsclite-dev

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev libpcsclite-dev pcscd

# Fedora
sudo dnf install gcc pkgconf-pkg-config gtk4-devel libadwaita-devel pcsc-lite-devel

# Build
cd rust
cargo build --release
./target/release/gosh-authenticator
```

## Troubleshooting

**PC/SC service not running:** Start with `sudo systemctl start pcscd`

**YubiKey not detected:** Check with `pcsc_scan` or verify pcscd is running

## License

GPL-3.0-or-later
