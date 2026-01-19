# Gosh Yubico Authenticator for Linux

A GTK 4 + Rust desktop application for managing OATH (TOTP/HOTP) credentials stored on YubiKey devices.

## Features

- List OATH credentials from YubiKey
- Generate TOTP codes with 30-second countdown
- Generate HOTP codes with manual increment
- Add new credentials (manual entry)
- Delete credentials
- Support for password-protected YubiKeys
- Support for touch-required credentials
- Copy codes to clipboard

## Requirements

### Runtime Dependencies

- **pcscd** - PC/SC Smart Card Daemon (required for YubiKey communication)
- GTK 4.12+
- libadwaita 1.4+

### Build Dependencies

- Rust 1.70+
- pkg-config
- libpcsclite-dev
- libgtk-4-dev
- libadwaita-1-dev

## Installation

### Ubuntu/Debian

```bash
# Install runtime dependencies
sudo apt install pcscd

# Install build dependencies
sudo apt install pkgconf libpcsclite-dev libgtk-4-dev libadwaita-1-dev

# Build
cargo build --release
```

### Fedora

```bash
# Install runtime dependencies
sudo dnf install pcsc-lite

# Install build dependencies
sudo dnf install pkgconf pcsc-lite-devel gtk4-devel libadwaita-devel

# Build
cargo build --release
```

### Arch Linux

```bash
# Install runtime dependencies
sudo pacman -S pcsclite

# Install build dependencies
sudo pacman -S pkgconf gtk4 libadwaita

# Build
cargo build --release
```

## Running

1. **Start the PC/SC daemon** (required):

   ```bash
   sudo systemctl start pcscd
   ```

   Or enable it to start automatically:

   ```bash
   sudo systemctl enable pcscd
   ```

2. **Insert your YubiKey**

3. **Run the application**:

   ```bash
   cargo run --release
   ```

   Or if installed:

   ```bash
   gosh-authenticator
   ```

## Troubleshooting

### "PC/SC service not running"

The PC/SC daemon is not running. Start it with:

```bash
sudo systemctl start pcscd
```

### "No YubiKey Connected"

- Ensure your YubiKey is inserted
- Check that pcscd is running
- Verify your user has permission to access USB devices (you may need to be in the `plugdev` group)

### YubiKey not detected

Try checking if pcscd sees your device:

```bash
pcsc_scan
```

## License

GPL-3.0-or-later
