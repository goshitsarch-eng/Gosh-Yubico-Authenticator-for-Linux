Name:           gosh-authenticator
Version:        2.0.0
Release:        1%{?dist}
Summary:        Qt 6 / Kirigami app for managing OATH credentials on YubiKey devices
License:        GPL-3.0-or-later
URL:            https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cmake
BuildRequires:  gcc-c++
BuildRequires:  cmake(Qt6Core)
BuildRequires:  cmake(Qt6Gui)
BuildRequires:  cmake(Qt6Network)
BuildRequires:  cmake(Qt6Qml)
BuildRequires:  cmake(Qt6Quick)
BuildRequires:  cmake(Qt6QuickControls2)
BuildRequires:  cmake(Qt6Widgets)
BuildRequires:  cmake(Qt6Test)
BuildRequires:  cmake(KF6ColorScheme)
BuildRequires:  cmake(KF6KIO)
BuildRequires:  cmake(ZXing)
BuildRequires:  pcsc-lite-devel

Requires:       pcsc-lite
Requires:       kf6-kirigami
Requires:       kf6-qqc2-desktop-style
Requires:       qt6-qtsvg

%description
Gosh Yubico Authenticator is a native Qt 6 / Kirigami Linux desktop
application for managing OATH (TOTP/HOTP) credentials on YubiKey devices.
It integrates with KDE Plasma and follows the system color scheme, with
optional forced light and dark modes.

%prep
%autosetup

%build
%cmake -DBUILD_TESTING=ON
%cmake_build

%check
%ctest

%install
%cmake_install

%files
%license LICENSE
%doc README.md
%{_bindir}/gosh-authenticator
%{_datadir}/applications/com.goshapps.YubicoAuthenticator.desktop
%{_datadir}/icons/hicolor/scalable/apps/com.goshapps.YubicoAuthenticator.svg
%{_datadir}/icons/hicolor/64x64/apps/com.goshapps.YubicoAuthenticator.png
%{_datadir}/icons/hicolor/128x128/apps/com.goshapps.YubicoAuthenticator.png
%{_datadir}/metainfo/com.goshapps.YubicoAuthenticator.metainfo.xml

%changelog
* Sun Aug 30 2026 Gosh OS <goshitsarch-eng@users.noreply.github.com> - 2.0.0-1
- Complete rewrite as a native Qt 6 / Kirigami (KDE Frameworks 6) application
- Follow the system color scheme with forced light/dark modes via KColorScheme
- Open QR images from network shares in-app through KIO
- Drop all GTK 4 / libadwaita and Rust code and dependencies

* Mon Aug 24 2026 Gosh OS <goshitsarch-eng@users.noreply.github.com> - 1.2.1-1
- Fail closed when the OATH applet omits or malforms its device ID
- Remove unnecessary Flatpak network and broad device permissions

* Thu Aug 20 2026 Builder <builder@localhost> - 1.2.0-1
- Rewrite as native GTK 4 / Adwaita application
- Add Flatpak packaging (GNOME 50 runtime)
