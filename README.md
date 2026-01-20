# Gosh Yubico Authenticator for Linux

A Qt 6 + Rust desktop application for managing OATH (TOTP/HOTP) credentials stored on YubiKey devices.

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
- Qt 6 (QtBase)

### Build Dependencies

- Rust 1.70+
- CMake 3.20+
- Qt 6 development packages
- libpcsclite-dev

## Installation

> Note: The build uses CMake with Corrosion to compile the Rust core and link it into the Qt app.

### Ubuntu/Debian

```bash
# Install runtime dependencies
sudo apt install pcscd

# Install build dependencies
sudo apt install cmake build-essential pkgconf libpcsclite-dev qt6-base-dev

# Build
cmake -S . -B build
cmake --build build --config Release

# Install (optional)
cmake --install build
```

### Fedora

```bash
# Install runtime dependencies
sudo dnf install pcsc-lite

# Install build dependencies
sudo dnf install cmake gcc-c++ pkgconf pcsc-lite-devel qt6-qtbase-devel

# Build
cmake -S . -B build
cmake --build build --config Release

# Install (optional)
cmake --install build
```

### Arch Linux

```bash
# Install runtime dependencies
sudo pacman -S pcsclite

# Install build dependencies
sudo pacman -S cmake base-devel pkgconf qt6-base

# Build
cmake -S . -B build
cmake --build build --config Release

# Install (optional)
cmake --install build
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
   ./build/gosh-authenticator
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
