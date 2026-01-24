Name:           gosh-authenticator
Version:        1.0.0
Release:        1%{?dist}
Summary:        A desktop application for managing OATH credentials on YubiKey devices
License:        MIT
URL:            https://github.com/gosh/Gosh-Yubico-Authenticator-for-Linux
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  gcc-c++
BuildRequires:  cmake
BuildRequires:  ninja-build
BuildRequires:  pkgconfig(gtk+-3.0)
BuildRequires:  pcsc-lite-devel
BuildRequires:  clang
BuildRequires:  rust
BuildRequires:  cargo

Requires:       pcsc-lite
Requires:       gtk3

# Disable debug package generation (Flutter builds don't include debug symbols)
%global debug_package %{nil}

# Disable RPATH checks for Flutter-built libraries
%global __brp_check_rpaths %{nil}

%description
Gosh Yubico Authenticator is a Linux desktop application for managing OATH
(TOTP/HOTP) credentials on YubiKey devices. It features a modern Flutter UI
with a high-performance Rust core for direct hardware communication.

%prep
%autosetup

%build
# Build Rust library
cd rust
cargo build --release
cd ..

# Note: Flutter build requires flutter SDK in PATH
# This spec assumes flutter is available during build
# Clear RPM-specific compiler flags that are incompatible with clang
unset CFLAGS CXXFLAGS LDFLAGS
cd flutter_app
if command -v flutter &> /dev/null; then
    flutter pub get
    flutter build linux --release
else
    echo "ERROR: Flutter SDK not found in PATH"
    echo "Please install Flutter SDK and add it to PATH before building"
    exit 1
fi
cd ..

%install
# Create necessary directories
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_libdir}/%{name}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/scalable/apps

# Install the Flutter application
cp -r flutter_app/build/linux/x64/release/bundle/* %{buildroot}%{_libdir}/%{name}/

# Create wrapper script
cat > %{buildroot}%{_bindir}/gosh-authenticator << 'EOF'
#!/bin/bash
exec /usr/lib64/gosh-authenticator/gosh_yubikey_manager "$@"
EOF
chmod +x %{buildroot}%{_bindir}/gosh-authenticator

# Install desktop file
cat > %{buildroot}%{_datadir}/applications/gosh-authenticator.desktop << 'EOF'
[Desktop Entry]
Name=Gosh Authenticator
Comment=Manage OATH credentials on YubiKey devices
Exec=gosh-authenticator
Icon=gosh-authenticator
Terminal=false
Type=Application
Categories=Utility;Security;
Keywords=yubikey;authenticator;2fa;totp;hotp;
EOF

# Install icon
install -Dm644 icon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/gosh-authenticator.svg

%files
%{_bindir}/gosh-authenticator
%{_libdir}/%{name}/
%{_datadir}/applications/gosh-authenticator.desktop
%{_datadir}/icons/hicolor/scalable/apps/gosh-authenticator.svg

%post
/usr/bin/update-desktop-database &> /dev/null || :
/usr/bin/gtk-update-icon-cache %{_datadir}/icons/hicolor &> /dev/null || :

%postun
/usr/bin/update-desktop-database &> /dev/null || :
/usr/bin/gtk-update-icon-cache %{_datadir}/icons/hicolor &> /dev/null || :

%changelog
* Thu Jan 23 2026 Builder <builder@localhost> - 1.0.0-1
- Initial RPM package
