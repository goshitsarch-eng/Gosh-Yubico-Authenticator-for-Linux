Name:           gosh-authenticator
Version:        1.2.1
Release:        1%{?dist}
Summary:        Native GTK 4 app for managing OATH credentials on YubiKey devices
License:        GPL-3.0-or-later
URL:            https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  pkgconfig(libadwaita-1)
BuildRequires:  pcsc-lite-devel
BuildRequires:  rust
BuildRequires:  cargo

Requires:       pcsc-lite
Requires:       gtk4
Requires:       libadwaita

%description
Gosh Yubico Authenticator is a native GTK 4 / Adwaita Linux desktop
application for managing OATH (TOTP/HOTP) credentials on YubiKey devices.

%prep
%autosetup

%build
cd rust
cargo build --release --locked

%install
install -D -m 0755 rust/target/release/gosh-authenticator \
  %{buildroot}%{_bindir}/gosh-authenticator
install -D -m 0644 data/applications/com.goshapps.YubicoAuthenticator.desktop \
  %{buildroot}%{_datadir}/applications/com.goshapps.YubicoAuthenticator.desktop
install -D -m 0644 data/icons/hicolor/scalable/apps/com.goshapps.YubicoAuthenticator.svg \
  %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/com.goshapps.YubicoAuthenticator.svg
install -D -m 0644 data/icons/hicolor/64x64/apps/com.goshapps.YubicoAuthenticator.png \
  %{buildroot}%{_datadir}/icons/hicolor/64x64/apps/com.goshapps.YubicoAuthenticator.png
install -D -m 0644 data/icons/hicolor/128x128/apps/com.goshapps.YubicoAuthenticator.png \
  %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/com.goshapps.YubicoAuthenticator.png
install -D -m 0644 data/metainfo/com.goshapps.YubicoAuthenticator.metainfo.xml \
  %{buildroot}%{_datadir}/metainfo/com.goshapps.YubicoAuthenticator.metainfo.xml

%files
%license LICENSE
%doc README.md
%{_bindir}/gosh-authenticator
%{_datadir}/applications/com.goshapps.YubicoAuthenticator.desktop
%{_datadir}/icons/hicolor/scalable/apps/com.goshapps.YubicoAuthenticator.svg
%{_datadir}/icons/hicolor/64x64/apps/com.goshapps.YubicoAuthenticator.png
%{_datadir}/icons/hicolor/128x128/apps/com.goshapps.YubicoAuthenticator.png
%{_datadir}/metainfo/com.goshapps.YubicoAuthenticator.metainfo.xml

%post
/usr/bin/update-desktop-database &> /dev/null || :
/usr/bin/gtk-update-icon-cache %{_datadir}/icons/hicolor &> /dev/null || :

%postun
/usr/bin/update-desktop-database &> /dev/null || :
/usr/bin/gtk-update-icon-cache %{_datadir}/icons/hicolor &> /dev/null || :

%changelog
* Mon Aug 24 2026 Gosh OS <goshitsarch-eng@users.noreply.github.com> - 1.2.1-1
- Fail closed when the OATH applet omits or malforms its device ID
- Remove unnecessary Flatpak network and broad device permissions

* Thu Aug 20 2026 Builder <builder@localhost> - 1.2.0-1
- Rewrite as native GTK 4 / Adwaita application
- Add Flatpak packaging (GNOME 50 runtime)
