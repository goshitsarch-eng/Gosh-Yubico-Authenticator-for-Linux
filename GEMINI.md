# Gosh Yubico Authenticator for Linux

## Project Overview

**Gosh Yubico Authenticator** is a Linux desktop application for managing OATH (TOTP/HOTP) credentials on YubiKey devices. 

The project employs a hybrid architecture:
- **Core Logic:** Written in **Rust** (`rust/`) for performance, safety, and direct hardware access via PC/SC.
- **User Interface:** Built with **Flutter** (`flutter_app/`) for a responsive, modern desktop UI.

## Architecture & Components

### 1. Rust Core (`rust/`)
*   **Role:** Handles all communication with the YubiKey (APDU commands), implements OATH protocols (TOTP/HOTP), and manages crypto operations.
*   **Key Crate:** `gosh_authenticator_core`
*   **Dependencies:** `pcsc` (smart card access), `hmac`, `sha1`, `sha2`.
*   **Interface:** Exposes a C-compatible FFI (defined in `include/gosh_ffi.h`) for consumption by the Flutter app.

### 2. Flutter App (`flutter_app/`)
*   **Role:** The primary user interface.
*   **Tech Stack:** Dart, Flutter, Riverpod (state management), FFI (to communicate with Rust).
*   **Integration:** The Linux build process (`flutter_app/linux/CMakeLists.txt`) automatically invokes `cargo build` to compile the Rust core and bundles the resulting `libgosh_authenticator_core.so`.
*   **Key Paths:**
    *   `lib/src/ffi/`: Dart FFI bindings.
    *   `lib/src/providers/`: State management logic.

## Build & Development

### Prerequisites
*   **System:** Linux (Ubuntu/Debian, Fedora, Arch)
*   **Runtime:** `pcscd` (PC/SC Smart Card Daemon)
*   **Build Tools:**
    *   Rust Toolchain (latest stable)
    *   Flutter SDK
    *   `libpcsclite-dev` (or equivalent)
    *   `clang`, `cmake`, `ninja-build`, `pkg-config`, `libgtk-3-dev` (Standard Flutter Linux requirements)

### Building and Running
The development workflow is standard for a Flutter desktop app.

```bash
cd flutter_app
flutter pub get
flutter run -d linux
```
*Note: The build scripts automatically compile the Rust library and place it in the bundle.*

## Key Configuration Files
*   **`rust/Cargo.toml`**: Rust dependencies and workspace config.
*   **`flutter_app/pubspec.yaml`**: Flutter dependencies and asset configuration.
*   **`flutter_app/linux/CMakeLists.txt`**: The glue code that builds the Rust library when building the Flutter app.
