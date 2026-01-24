# Building RPM Package for Gosh Authenticator

## Prerequisites

### 1. Install RPM Build Tools
```bash
sudo dnf install rpm-build rpmdevtools gcc gcc-c++ cmake ninja-build \
                 gtk3-devel pcsc-lite-devel clang rust cargo
```

### 2. Install Flutter SDK
Download and install Flutter from: https://docs.flutter.dev/get-started/install/linux

Add Flutter to your PATH:
```bash
export PATH="$PATH:/path/to/flutter/bin"
```

## Building the RPM

Simply run the build script:
```bash
./build-rpm.sh
```

This will:
1. Create the RPM build environment in `~/rpmbuild`
2. Create a source tarball
3. Build the RPM package

## Installing the RPM

After a successful build:
```bash
sudo dnf install ~/rpmbuild/RPMS/x86_64/gosh-authenticator-1.0.0-*.rpm
```

Or for testing:
```bash
sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/gosh-authenticator-1.0.0-*.rpm
```

## Manual Build Process

If you prefer to build manually:

1. Setup RPM build environment:
```bash
rpmdev-setuptree
```

2. Create source tarball:
```bash
cd ..
tar --exclude='.git' --exclude='flutter_app/build' --exclude='rust/target' \
    -czf ~/rpmbuild/SOURCES/gosh-authenticator-1.0.0.tar.gz \
    Gosh-Yubico-Authenticator-for-Linux/
```

3. Copy spec file:
```bash
cp gosh-authenticator.spec ~/rpmbuild/SPECS/
```

4. Build RPM:
```bash
cd ~/rpmbuild/SPECS
rpmbuild -bb gosh-authenticator.spec
```

## Package Contents

The RPM will install:
- Binary wrapper: `/usr/bin/gosh-authenticator`
- Application files: `/usr/lib64/gosh-authenticator/`
- Desktop entry: `/usr/share/applications/gosh-authenticator.desktop`
- Icon: `/usr/share/icons/hicolor/scalable/apps/gosh-authenticator.svg`

## Running the Application

After installation, you can run from:
- Application menu (search for "Gosh Authenticator")
- Terminal: `gosh-authenticator`

## Dependencies

Runtime dependencies are automatically handled:
- pcsc-lite (for smart card access)
- gtk3 (for GUI)
