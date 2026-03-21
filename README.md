# Gosh Authenticator

A pure Rust desktop application for managing OATH (TOTP/HOTP) credentials on YubiKey devices. Built with the [Iced](https://github.com/iced-rs/iced) GUI framework.

## Features

- **TOTP & HOTP** credential management on YubiKey hardware
- **Multiple algorithms:** SHA-1, SHA-256, SHA-512
- **6/7/8-digit codes** with configurable TOTP periods
- **Touch-required credentials** for enhanced security
- **YubiKey password protection** (set, change, remove)
- **QR code import** from image files (otpauth:// URI parsing)
- **Clipboard integration** with configurable auto-clear timeout
- **Cross-platform:** Linux, Windows, macOS
- **Dark/Light/System theme** support
- **Search and filter** credentials

## Screenshots

![Gosh YubiKey Manager – Credentials View](screenshots/img1.png)

## Quick Start

### Prerequisites

Install PC/SC Smart Card Daemon for YubiKey communication:

```bash
# Fedora/RHEL
sudo dnf install pcsc-lite pcsc-lite-devel

# Ubuntu/Debian
sudo apt install pcscd libpcsclite-dev

# Arch Linux
sudo pacman -S pcsclite

# macOS (via Homebrew)
brew install pcsc-lite

# Windows: PC/SC is built into the OS (winscard)
```

Enable and start the daemon (Linux):
```bash
sudo systemctl enable --now pcscd
```

### Run from Release

```bash
# Download and extract the release, then:
./gosh-authenticator

# Or install the RPM/DEB package
```

## Building from Source

**Requirements:** Rust 1.70+ and platform PC/SC development headers.

```bash
# Install build dependencies
sudo dnf install pcsc-lite-devel      # Fedora
sudo apt install libpcsclite-dev      # Ubuntu/Debian
sudo pacman -S pcsclite               # Arch

# Clone and build
git clone https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux.git
cd Gosh-Yubico-Authenticator-for-Linux
cargo build --release

# Run
./target/release/gosh-authenticator
```

### Cross-Platform Notes

| Platform | PC/SC Library | Notes |
|----------|--------------|-------|
| Linux    | `libpcsclite-dev` | Requires `pcscd` running |
| macOS    | Built-in PCSC.framework | Works out of the box |
| Windows  | Built-in WinSCard | Works out of the box |

## Architecture

This is a **pure Rust** application with no Flutter or other non-Rust dependencies:

```
src/
├── main.rs                    # Entry point (Iced application)
├── core/
│   ├── credential.rs          # Credential types, base32, otpauth parsing
│   └── yubikey/
│       ├── apdu.rs            # ISO 7816-4 APDU protocol
│       ├── connection.rs      # PC/SC YubiKey connection
│       ├── oath.rs            # OATH session operations
│       └── error.rs           # Error types
├── services/
│   ├── yubikey_service.rs     # Background worker thread
│   ├── clipboard.rs           # Clipboard with auto-clear
│   └── settings.rs            # Persistent app settings
└── ui/
    ├── app.rs                 # Main app state & message handling
    ├── theme.rs               # Color palette
    ├── screens/
    │   ├── home.rs            # Credentials list with countdown
    │   ├── add_credential.rs  # Add credential form + QR import
    │   ├── settings.rs        # Theme, clipboard, password settings
    │   ├── key_info.rs        # YubiKey device info
    │   └── about.rs           # About page
    └── widgets/
        └── credential_card.rs # Credential card component
```

### Key Dependencies

- **[iced](https://github.com/iced-rs/iced)** - Cross-platform GUI framework
- **[pcsc](https://crates.io/crates/pcsc)** - PC/SC smart card communication
- **[arboard](https://crates.io/crates/arboard)** - Cross-platform clipboard
- **[rqrr](https://crates.io/crates/rqrr)** - QR code detection and decoding
- **[rfd](https://crates.io/crates/rfd)** - Native file dialogs

## Troubleshooting

**PC/SC service not running:** `sudo systemctl start pcscd`

**YubiKey not detected:** Check with `pcsc_scan` or verify `pcscd` is running

**Build fails on missing pcsc:** Install `libpcsclite-dev` (Debian) or `pcsc-lite-devel` (Fedora)

## License

GPL-3.0-or-later
