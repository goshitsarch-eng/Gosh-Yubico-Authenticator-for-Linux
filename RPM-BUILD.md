# Building RPM Package for Gosh Authenticator

## Prerequisites

```bash
sudo dnf install rpm-build rpmdevtools cmake gcc-c++ \
                 qt6-qtbase-devel qt6-qtdeclarative-devel \
                 kf6-kcolorscheme-devel kf6-kio-devel \
                 zxing-cpp-devel pcsc-lite-devel
```

## Building the RPM

```bash
./build-rpm.sh
```

This will:
1. Create the RPM build environment in `~/rpmbuild`
2. Create a source tarball
3. Build the RPM package (including running the unit tests)

## Installing the RPM

```bash
sudo dnf install ~/rpmbuild/RPMS/x86_64/gosh-authenticator-2.0.0-*.rpm
```

## Package Contents

The RPM installs:
- Binary: `/usr/bin/gosh-authenticator`
- Desktop entry: `/usr/share/applications/com.goshapps.YubicoAuthenticator.desktop`
- Icon: `/usr/share/icons/hicolor/scalable/apps/com.goshapps.YubicoAuthenticator.svg`
- AppStream metainfo: `/usr/share/metainfo/com.goshapps.YubicoAuthenticator.metainfo.xml`

For a sandboxed install, prefer the Flatpak:

```bash
./build-flatpak.sh
flatpak run com.goshapps.YubicoAuthenticator
```

## Dependencies

Runtime dependencies:
- pcsc-lite (for smart card access)
- Qt 6 (base, declarative)
- KDE Frameworks 6: Kirigami, qqc2-desktop-style, KColorScheme, KIO
- zxing-cpp (QR decoding)
