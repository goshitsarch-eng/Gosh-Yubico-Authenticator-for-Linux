# Gosh Yubico Authenticator for Linux

A Flutter + Rust desktop application for managing OATH (TOTP/HOTP) credentials on YubiKey devices.

## Features

Manage OATH credentials with TOTP/HOTP code generation, password-protected YubiKeys, touch-required credentials, and clipboard integration.

## Screenshots

![Gosh YubiKey Manager – Credentials View](screenshots/img1.png)


## Quick Start

**Prerequisites:** Install `pcscd` (PC/SC Smart Card Daemon) for YubiKey communication.

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
# Download and extract the release tarball, then:
./gosh_yubikey_manager

# Or install the RPM package:
sudo dnf install gosh-authenticator-*.rpm
```

## Building from Source

**Build dependencies:** Rust 1.70+, Flutter 3.x, libpcsclite-dev

```bash
# Install dependencies
sudo dnf install pcsc-lite-devel  # Fedora
sudo apt install libpcsclite-dev  # Ubuntu

# Build
cd flutter_app
flutter build linux --release
./build/linux/x64/release/bundle/gosh_yubikey_manager
```

**System-wide installation:**
```bash
cd flutter_app/build/linux/x64/release
sudo cmake --install . --prefix /usr/local
sudo update-desktop-database /usr/local/share/applications
sudo gtk-update-icon-cache /usr/local/share/icons/hicolor
```

## Troubleshooting

**PC/SC service not running:** Start with `sudo systemctl start pcscd`

**YubiKey not detected:** Check with `pcsc_scan` or verify pcscd is running

**Wrong icon on KDE Wayland:** Fixed in recent versions. Log out/in to refresh the desktop cache if needed.

## License

GPL-3.0-or-later
