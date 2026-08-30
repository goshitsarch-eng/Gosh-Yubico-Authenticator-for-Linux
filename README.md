# Gosh Yubico Authenticator for Linux

A native Qt 6 / Kirigami (KDE Frameworks 6) desktop application for managing
OATH (TOTP/HOTP) credentials on YubiKey devices.

Gosh Yubico Authenticator is an independent application and is not affiliated
with or endorsed by Yubico. Yubico and YubiKey are trademarks of Yubico AB.

## Features

Manage OATH credentials with TOTP/HOTP code generation, password-protected
YubiKeys, touch-required credentials, clipboard auto-clear, QR import (from
local files or network shares), and system / light / dark color schemes. The
app is built with Kirigami and looks and behaves like a native KDE
application on Plasma, while remaining fully usable on any desktop.

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

**Install the Flatpak (recommended):**

```bash
# From a release bundle:
flatpak install --user ./com.goshapps.YubicoAuthenticator.flatpak
flatpak run com.goshapps.YubicoAuthenticator

# Or build and install from this repository:
./build-flatpak.sh
```

The Flatpak talks to the **host** `pcscd` over the `pcsc` socket. Keep the
daemon running on the host; do not expect a daemon inside the sandbox.

**Other packages:**

```bash
# Distro packages:
sudo dnf install gosh-authenticator-*.rpm         # Fedora
sudo apt install ./gosh-authenticator_*.deb       # Debian 13+/Ubuntu 24.10+

# From a release tarball (needs Qt 6 + Kirigami installed):
./gosh-authenticator
```

## Building from Source

**Build dependencies:** CMake 3.20+, a C++17 compiler, Qt 6.4+ (base,
declarative), KDE Frameworks 6 (Kirigami, KColorScheme, KIO), zxing-cpp, and
libpcsclite.

```bash
# Fedora
sudo dnf install cmake gcc-c++ qt6-qtbase-devel qt6-qtdeclarative-devel \
  kf6-kirigami-devel kf6-kcolorscheme-devel kf6-kio-devel \
  zxing-cpp-devel pcsc-lite-devel

# Debian 13 (trixie) / Ubuntu 24.10+
sudo apt install cmake g++ qt6-base-dev qt6-declarative-dev \
  qml6-module-org-kde-kirigami libkf6colorscheme-dev libkf6kio-dev \
  libzxing-dev libpcsclite-dev pcscd

# Build and run
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
./build/gosh-authenticator
```

Run the unit tests with `ctest --test-dir build`.

KColorScheme, KIO, and zxing-cpp are optional at build time: without them the
app still builds, with palette-based theming, local-file-only QR import, and
no QR scanning respectively. Install them for the full experience.

**Flatpak from source:**

```bash
sudo apt install flatpak flatpak-builder elfutils
./build-flatpak.sh
```

`./build-flatpak.sh` installs the KDE 6.9 SDK and Platform from Flathub,
builds `com.goshapps.YubicoAuthenticator`, installs it for the current user,
and writes a `.flatpak` bundle under `dist/`.

## Troubleshooting

**PC/SC service not running:** Start with `sudo systemctl start pcscd`

**YubiKey not detected:** Check with `pcsc_scan` or verify pcscd is running

**Unthemed look outside KDE:** Install `qqc2-desktop-style` (Fedora:
`kf6-qqc2-desktop-style`, Debian: `qml6-module-org-kde-desktop`)

## License

GPL-3.0-or-later
