Name:           gosh-authenticator
# Cargo's release profile already strips this binary; no separate debug RPM.
%global debug_package %{nil}
Version:        2.0.0
Release:        1%{?dist}
Summary:        Dioxus Desktop app for managing OATH credentials on YubiKey devices
License:        GPL-3.0-or-later
URL:            https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  pkgconfig(gtk+-3.0)
BuildRequires:  pkgconfig(webkit2gtk-4.1)
BuildRequires:  libxdo-devel
BuildRequires:  pcsc-lite-devel
BuildRequires:  rust
BuildRequires:  cargo

Requires:       pcsc-lite
Requires:       gtk3
Requires:       webkit2gtk4.1
Requires:       libX11
Requires:       libxdo

%description
Gosh Yubico Authenticator is a Rust / Dioxus Desktop
application for managing OATH (TOTP/HOTP) credentials on YubiKey devices.

%prep
%autosetup

%build
cd rust
# Reuse the existing target directory when called by build-rpm.sh, then stage
# the verified output in the source tree for RPM's install phase.
cargo build --release --locked
if [ -n "$CARGO_TARGET_DIR" ]; then
  mkdir -p target/release
  cp "$CARGO_TARGET_DIR/release/gosh-authenticator" target/release/gosh-authenticator
fi

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
%license LICENSE THIRD_PARTY_LICENSES.html
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
