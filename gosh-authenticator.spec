Name:           gosh-authenticator
Version:        2.0.0
Release:        1%{?dist}
Summary:        A pure Rust desktop application for managing OATH credentials on YubiKey devices
License:        GPL-3.0-or-later
URL:            https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  pcsc-lite-devel
BuildRequires:  rust
BuildRequires:  cargo

Requires:       pcsc-lite

%global debug_package %{nil}

%description
Gosh Authenticator is a pure Rust cross-platform desktop application for
managing OATH (TOTP/HOTP) credentials on YubiKey devices. Built with the
Iced GUI framework for a native look and feel.

%prep
%autosetup

%build
cargo build --release

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/scalable/apps

# Install binary
install -Dm755 target/release/gosh-authenticator %{buildroot}%{_bindir}/gosh-authenticator

# Install desktop file
cat > %{buildroot}%{_datadir}/applications/com.github.gosh.gosh_authenticator.desktop << EOF
[Desktop Entry]
Type=Application
Name=Gosh Authenticator
Comment=YubiKey OATH credential manager
Exec=gosh-authenticator
Icon=com.github.gosh.gosh_authenticator
Terminal=false
Categories=Utility;Security;
Keywords=YubiKey;TOTP;HOTP;OTP;Authenticator;2FA;
EOF

# Install icon
install -Dm644 icon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/com.github.gosh.gosh_authenticator.svg

%files
%{_bindir}/gosh-authenticator
%{_datadir}/applications/com.github.gosh.gosh_authenticator.desktop
%{_datadir}/icons/hicolor/scalable/apps/com.github.gosh.gosh_authenticator.svg

%post
/usr/bin/update-desktop-database &> /dev/null || :
/usr/bin/gtk-update-icon-cache %{_datadir}/icons/hicolor &> /dev/null || :

%postun
/usr/bin/update-desktop-database &> /dev/null || :
/usr/bin/gtk-update-icon-cache %{_datadir}/icons/hicolor &> /dev/null || :

%changelog
* Sat Mar 21 2026 Builder <builder@localhost> - 2.0.0-1
- Migrate to pure Rust with Iced GUI (no more Flutter)
- Cross-platform support (Linux, Windows, macOS)

* Sat Jan 24 2026 Builder <builder@localhost> - 1.2.0-1
- Update to v1.2.0

* Fri Jan 23 2026 Builder <builder@localhost> - 1.0.0-1
- Initial RPM package
