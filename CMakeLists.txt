cmake_minimum_required(VERSION 3.20)

project(gosh_authenticator VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC OFF)

find_package(Qt6 REQUIRED COMPONENTS Widgets Gui)
find_package(PkgConfig REQUIRED)
pkg_check_modules(PCSC REQUIRED libpcsclite)
qt_standard_project_setup()

find_program(CARGO_EXECUTABLE cargo REQUIRED)
set(RUST_LIB ${CMAKE_CURRENT_SOURCE_DIR}/target/release/libgosh_authenticator_core.a)

add_custom_command(
    OUTPUT ${RUST_LIB}
    COMMAND ${CARGO_EXECUTABLE} build --release --manifest-path ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    BYPRODUCTS ${RUST_LIB}
    COMMENT "Building Rust core (release)"
)

add_custom_target(rust_core ALL DEPENDS ${RUST_LIB})

add_library(gosh_authenticator_core STATIC IMPORTED GLOBAL)
set_target_properties(gosh_authenticator_core PROPERTIES IMPORTED_LOCATION ${RUST_LIB})
add_dependencies(gosh_authenticator_core rust_core)

add_executable(gosh-authenticator-app
    qt/main.cpp
    qt/mainwindow.cpp
    qt/gosh_client.cpp
    qt/credential_row_widget.cpp
    qt/add_credential_dialog.cpp
    qt/password_dialog.cpp
    qt/toast_overlay.cpp
    qt/resources.qrc
)

set_target_properties(gosh-authenticator-app PROPERTIES OUTPUT_NAME "gosh-authenticator")

target_include_directories(gosh-authenticator-app PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_SOURCE_DIR}/qt
)

target_compile_definitions(gosh-authenticator-app PRIVATE
    VERSION_STRING="${PROJECT_VERSION}"
)

target_link_libraries(gosh-authenticator-app PRIVATE
    Qt6::Widgets
    Qt6::Gui
    gosh_authenticator_core
    ${PCSC_LIBRARIES}
)

target_include_directories(gosh-authenticator-app PRIVATE ${PCSC_INCLUDE_DIRS})
target_link_directories(gosh-authenticator-app PRIVATE ${PCSC_LIBRARY_DIRS})

install(TARGETS gosh-authenticator-app RUNTIME DESTINATION bin)
install(FILES data/com.github.gosh.authenticator.desktop
        DESTINATION share/applications)
install(FILES data/icons/hicolor/scalable/apps/com.github.gosh.authenticator.svg
        DESTINATION share/icons/hicolor/scalable/apps)
install(FILES data/icons/hicolor/symbolic/apps/com.github.gosh.authenticator-symbolic.svg
        DESTINATION share/icons/hicolor/symbolic/apps)
install(FILES README.md DESTINATION share/doc/gosh-authenticator)

set(CPACK_PACKAGE_NAME "gosh-authenticator")
set(CPACK_PACKAGE_VENDOR "Gosh")
set(CPACK_PACKAGE_CONTACT "Gosh Team")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Manage OATH credentials on YubiKey devices")
set(CPACK_PACKAGE_VERSION "${PROJECT_VERSION}")
set(CPACK_PACKAGE_LICENSE "GPL-3.0-or-later")

set(CPACK_DEBIAN_PACKAGE_MAINTAINER "Gosh Team")
set(CPACK_DEBIAN_PACKAGE_DEPENDS "pcscd, qt6-base")

set(CPACK_RPM_PACKAGE_REQUIRES "pcsc-lite, qt6-qtbase")

include(CPack)
