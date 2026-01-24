# Gosh Yubico Authenticator for Linux

A Flutter + Rust desktop application for managing OATH (TOTP/HOTP) credentials stored on YubiKey devices.

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

### Build Dependencies

- Rust 1.70+
- Flutter 3.x (with desktop support enabled)
- libpcsclite-dev

## Installation

> Note: The build uses Flutter's desktop tooling and compiles the Rust core as a shared library.

### System-Wide Installation

For proper desktop integration (especially on KDE Plasma Wayland), install to a system directory:

```bash
cd flutter_app
flutter build linux --release
cd build/linux/x64/release
sudo cmake --install . --prefix /usr/local
```

This will install:
- Binary: `/usr/local/bin/gosh_yubikey_manager`
- Libraries: `/usr/local/lib/`
- Desktop file: `/usr/local/share/applications/com.github.gosh.gosh_yubikey_manager.desktop`
- Icon: `/usr/local/share/icons/hicolor/scalable/apps/com.github.gosh.gosh_yubikey_manager.svg`

After installation, update the desktop database:
```bash
sudo update-desktop-database /usr/local/share/applications
sudo gtk-update-icon-cache /usr/local/share/icons/hicolor
```

### Development Build (Local Bundle)

For development, the default build creates a relocatable bundle:

```bash
cd flutter_app
flutter build linux --release
./build/linux/x64/release/bundle/gosh_yubikey_manager
```

### Ubuntu/Debian

```bash
# Install runtime dependencies
sudo apt install pcscd

# Install build dependencies
sudo apt install libpcsclite-dev

# Build (Flutter desktop)
cd flutter_app
flutter build linux
```

### Fedora

```bash
# Install runtime dependencies
sudo dnf install pcsc-lite

# Install build dependencies
sudo dnf install pcsc-lite-devel

# Build (Flutter desktop)
cd flutter_app
flutter build linux
```

### Arch Linux

```bash
# Install runtime dependencies
sudo pacman -S pcsclite

# Install build dependencies
sudo pacman -S base-devel pcsclite

# Build (Flutter desktop)
cd flutter_app
flutter build linux
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
   ./flutter_app/build/linux/x64/release/bundle/gosh_authenticator
   ```

   Or if installed:

   ```bash
   ./flutter_app/build/linux/x64/release/bundle/gosh_authenticator
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

### Generic/Wrong Icon in Taskbar (Wayland)

If you're running the application on Wayland (especially KDE Plasma) and seeing a generic Wayland icon in the taskbar instead of the application icon:

**Solution**: This has been fixed in recent versions by setting the window role property. If you're building from source, make sure you have the latest code.

**Workaround for older versions**: 
1. Ensure the desktop file is properly installed in `~/.local/share/applications/` or `/usr/share/applications/`
2. The `StartupWMClass` in the desktop file should match: `com.github.gosh.gosh_yubikey_manager`
3. Log out and log back in to refresh the desktop environment's cache

## License

GPL-3.0-or-later
