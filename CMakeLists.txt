# SPDX-License-Identifier: GPL-3.0-or-later
cmake_minimum_required(VERSION 3.20)

project(gosh-authenticator VERSION 2.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(GNUInstallDirs)

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Network Qml Quick QuickControls2 Widgets)
find_package(PkgConfig REQUIRED)
pkg_check_modules(PCSC REQUIRED IMPORTED_TARGET libpcsclite)

# Optional integrations. All are present on a full KDE 6 stack and in the
# packaging below; the app builds and runs without them with reduced
# functionality (no QR-from-image, palette-based theming, local files only).
find_package(ZXing QUIET)
find_package(KF6ColorScheme QUIET)
find_package(KF6KIO QUIET)

qt_standard_project_setup()

configure_file(src/version.h.in ${CMAKE_CURRENT_BINARY_DIR}/generated/version.h @ONLY)

# Protocol / logic core, shared between the app and the unit tests.
add_library(authcore STATIC
    src/oath/apdu.cpp
    src/oath/oathsession.cpp
    src/otpauth.cpp
    src/serviceicons.cpp
)
target_include_directories(authcore PUBLIC src)
target_link_libraries(authcore PUBLIC
    Qt6::Core
    Qt6::Gui
    Qt6::Network
    PkgConfig::PCSC
)
if(ZXing_FOUND)
    target_compile_definitions(authcore PRIVATE HAVE_ZXING)
    target_link_libraries(authcore PRIVATE ZXing::ZXing)
endif()
if(KF6KIO_FOUND)
    target_compile_definitions(authcore PRIVATE HAVE_KIO)
    target_link_libraries(authcore PRIVATE KF6::KIOCore)
endif()

qt_add_executable(gosh-authenticator
    src/main.cpp
    src/appcontroller.cpp
    src/appsettings.cpp
    src/clipboardservice.cpp
    src/credentialmodel.cpp
    src/faviconcache.cpp
    src/thememanager.cpp
    src/yubikeyworker.cpp
)

qt_add_qml_module(gosh-authenticator
    URI com.goshapps.yubicoauthenticator
    VERSION 1.0
    RESOURCE_PREFIX /qt/qml
    QML_FILES
        src/qml/Main.qml
        src/qml/CredentialsPage.qml
        src/qml/CredentialDelegate.qml
        src/qml/AddCredentialPage.qml
        src/qml/KeyInfoPage.qml
        src/qml/SettingsPage.qml
        src/qml/UnlockDialog.qml
        src/qml/TouchDialog.qml
        src/qml/DeleteDialog.qml
        src/qml/ChangePasswordDialog.qml
        src/qml/IconPickerDialog.qml
)

target_include_directories(gosh-authenticator PRIVATE
    src
    ${CMAKE_CURRENT_BINARY_DIR}/generated
)
target_link_libraries(gosh-authenticator PRIVATE
    authcore
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::Widgets
)
if(KF6ColorScheme_FOUND)
    target_compile_definitions(gosh-authenticator PRIVATE HAVE_KCOLORSCHEME)
    target_link_libraries(gosh-authenticator PRIVATE KF6::ColorScheme)
endif()

install(TARGETS gosh-authenticator RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})
install(FILES data/applications/com.goshapps.YubicoAuthenticator.desktop
        DESTINATION ${CMAKE_INSTALL_DATADIR}/applications)
install(FILES data/icons/hicolor/scalable/apps/com.goshapps.YubicoAuthenticator.svg
        DESTINATION ${CMAKE_INSTALL_DATADIR}/icons/hicolor/scalable/apps)
install(FILES data/icons/hicolor/64x64/apps/com.goshapps.YubicoAuthenticator.png
        DESTINATION ${CMAKE_INSTALL_DATADIR}/icons/hicolor/64x64/apps)
install(FILES data/icons/hicolor/128x128/apps/com.goshapps.YubicoAuthenticator.png
        DESTINATION ${CMAKE_INSTALL_DATADIR}/icons/hicolor/128x128/apps)
install(FILES data/metainfo/com.goshapps.YubicoAuthenticator.metainfo.xml
        DESTINATION ${CMAKE_INSTALL_DATADIR}/metainfo)

# Unit tests
option(BUILD_TESTING "Build unit tests" ON)
if(BUILD_TESTING)
    enable_testing()
    find_package(Qt6 6.4 COMPONENTS Test)
    if(Qt6Test_FOUND)
        qt_add_executable(tst_oath tests/tst_oath.cpp)
        target_link_libraries(tst_oath PRIVATE authcore Qt6::Test)
        add_test(NAME tst_oath COMMAND tst_oath)
    endif()
endif()
