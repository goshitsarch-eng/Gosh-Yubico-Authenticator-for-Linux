# Building RPM Package for Gosh Authenticator

## Prerequisites

```bash
sudo dnf install rpm-build rpmdevtools gcc pkgconf-pkg-config \
                 gtk3-devel webkit2gtk4.1-devel libxdo-devel \
                 pcsc-lite-devel pcsc-lite ccid rust cargo
```

## Building the RPM

```bash
./build-rpm.sh
```

This will:
1. Create the RPM build environment in the checkout's `rpmbuild/` directory
2. Create a source tarball
3. Build the RPM package

## Installing the RPM

```bash
sudo dnf install rpmbuild/RPMS/x86_64/gosh-authenticator-2.0.0~alpha.1-*.rpm
sudo systemctl enable --now pcscd.socket
```

## Package Contents

The RPM installs:
- Binary: `/usr/bin/gosh-authenticator`
- Desktop entry: `/usr/share/applications/com.goshapps.YubicoAuthenticator.desktop`
- Icon: `/usr/share/icons/hicolor/scalable/apps/com.goshapps.YubicoAuthenticator.svg`
- AppStream metainfo: `/usr/share/metainfo/com.goshapps.YubicoAuthenticator.metainfo.xml`
- GPL license and third-party notices: `/usr/share/licenses/gosh-authenticator-VERSION/`

For a sandboxed install, prefer the Flatpak:

```bash
flatpak run com.goshapps.YubicoAuthenticator
```

## Dependencies

Runtime dependencies:
- pcsc-lite (for smart card access)
- GTK 3 (WebKitGTK and native menu integration)
- WebKitGTK 4.1
- libX11 and libxdo

The UI is Dioxus Desktop. See [BUILDING.md](BUILDING.md) for the pinned toolchain,
Flatpak installation and native build instructions. The locally built RPM was
extracted and tested on Debian 13 with a private dependency sysroot; native
Fedora installation and real-key QA remain required. Version 2.0.0-alpha.1 is a
migration prerelease; [QA.md](QA.md) records the open release gates.
