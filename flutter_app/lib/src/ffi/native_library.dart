import 'dart:ffi';
import 'dart:io';

import 'gosh_ffi_bindings.dart';

/// Loads the native Rust library for the current platform.
DynamicLibrary loadNativeLibrary() {
  if (Platform.isLinux) {
    return DynamicLibrary.open('libgosh_authenticator_core.so');
  } else if (Platform.isWindows) {
    return DynamicLibrary.open('gosh_authenticator_core.dll');
  } else if (Platform.isMacOS) {
    return DynamicLibrary.open('libgosh_authenticator_core.dylib');
  }
  throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
}

/// Global singleton for FFI bindings.
late final GoshFfiBindings goshFfi = GoshFfiBindings(loadNativeLibrary());
