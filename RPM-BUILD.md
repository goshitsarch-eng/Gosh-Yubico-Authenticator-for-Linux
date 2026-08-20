# Building RPM Package for Gosh Authenticator

## Prerequisites

```bash
sudo dnf install rpm-build rpmdevtools gcc pkgconf-pkg-config \
                 gtk4-devel libadwaita-devel pcsc-lite-devel rust cargo
```

## Building the RPM

```bash
./build-rpm.sh
```

This will:
1. Create the RPM build environment in `~/rpmbuild`
2. Create a source tarball
3. Build the RPM package

## Installing the RPM

```bash
sudo dnf install ~/rpmbuild/RPMS/x86_64/gosh-authenticator-1.2.0-*.rpm
```

## Package Contents

The RPM installs:
- Binary: `/usr/bin/gosh-authenticator`
- Desktop entry: `/usr/share/applications/com.github.gosh.gosh_yubikey_manager.desktop`
- Icon: `/usr/share/icons/hicolor/scalable/apps/com.github.gosh.gosh_yubikey_manager.svg`

## Dependencies

Runtime dependencies:
- pcsc-lite (for smart card access)
- gtk4
- libadwaita
