import 'dart:io';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:image/image.dart' as img;
import 'package:zxing_lib/common.dart';
import 'package:zxing_lib/qrcode.dart';
import 'package:zxing_lib/zxing.dart';

import '../ffi/gosh_event.dart';

/// Result of parsing an otpauth:// URI.
class OtpAuthCredential {
  final OathType type;
  final String? issuer;
  final String account;
  final String secret;
  final Algorithm algorithm;
  final int digits;
  final int? period;
  final int? counter;

  OtpAuthCredential({
    required this.type,
    this.issuer,
    required this.account,
    required this.secret,
    this.algorithm = Algorithm.sha1,
    this.digits = 6,
    this.period,
    this.counter,
  });
}

/// QR code scanner for desktop platforms.
///
/// Uses file picker to select an image containing a QR code,
/// then decodes it to extract otpauth:// credentials.
class QrScanner {
  /// Scan a QR code from a user-selected image file.
  ///
  /// Returns the decoded otpauth:// credential, or null if cancelled or failed.
  static Future<OtpAuthCredential?> scanFromFile() async {
    // Let user pick an image file
    final result = await FilePicker.platform.pickFiles(
      type: FileType.image,
      allowMultiple: false,
      dialogTitle: 'Select QR Code Image',
    );

    if (result == null || result.files.isEmpty) {
      return null;
    }

    final file = result.files.first;
    if (file.path == null) {
      throw QrScanException('Could not read selected file');
    }

    // Read and decode the image
    final bytes = await File(file.path!).readAsBytes();
    return _decodeQrFromBytes(bytes);
  }

  /// Decode a QR code from image bytes.
  static OtpAuthCredential? _decodeQrFromBytes(Uint8List bytes) {
    // Decode the image
    final image = img.decodeImage(bytes);
    if (image == null) {
      throw QrScanException('Could not decode image');
    }

    // Convert to luminance source for zxing
    final luminances = Int32List(image.width * image.height);
    for (int y = 0; y < image.height; y++) {
      for (int x = 0; x < image.width; x++) {
        final pixel = image.getPixel(x, y);
        // Convert to grayscale using standard formula
        final luminance =
            (0.299 * pixel.r + 0.587 * pixel.g + 0.114 * pixel.b).round();
        luminances[y * image.width + x] = luminance;
      }
    }

    // Create luminance source
    final source = RGBLuminanceSource(
      image.width,
      image.height,
      luminances,
    );

    // Try to decode QR code
    final binarizer = HybridBinarizer(source);
    final bitmap = BinaryBitmap(binarizer);

    try {
      final reader = QRCodeReader();
      final result = reader.decode(bitmap);

      final text = result.text;
      if (text == null || text.isEmpty) {
        throw QrScanException('QR code is empty');
      }

      return parseOtpAuthUri(text);
    } on NotFoundException {
      throw QrScanException('No QR code found in image');
    } on ReaderException {
      throw QrScanException('Could not read QR code');
    }
  }

  /// Parse an otpauth:// URI into credential data.
  ///
  /// Format: otpauth://TYPE/LABEL?PARAMETERS
  /// Example: otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP&issuer=Example
  static OtpAuthCredential parseOtpAuthUri(String uri) {
    final parsed = Uri.tryParse(uri);
    if (parsed == null) {
      throw QrScanException('Invalid URI format');
    }

    if (parsed.scheme != 'otpauth') {
      throw QrScanException(
          'Not an otpauth URI (got ${parsed.scheme}://)');
    }

    // Parse type (totp or hotp)
    final OathType type;
    switch (parsed.host.toLowerCase()) {
      case 'totp':
        type = OathType.totp;
        break;
      case 'hotp':
        type = OathType.hotp;
        break;
      default:
        throw QrScanException('Unknown OTP type: ${parsed.host}');
    }

    // Parse label (path contains issuer:account or just account)
    var path = parsed.path;
    if (path.startsWith('/')) {
      path = path.substring(1);
    }
    path = Uri.decodeComponent(path);

    String? issuer;
    String account;

    if (path.contains(':')) {
      final parts = path.split(':');
      issuer = parts[0].trim();
      account = parts.sublist(1).join(':').trim();
    } else {
      account = path.trim();
    }

    if (account.isEmpty) {
      throw QrScanException('Account name is required');
    }

    // Parse parameters
    final params = parsed.queryParameters;

    // Secret is required
    final secret = params['secret'];
    if (secret == null || secret.isEmpty) {
      throw QrScanException('Secret key is required');
    }

    // Issuer from parameter takes precedence
    if (params['issuer'] != null && params['issuer']!.isNotEmpty) {
      issuer = params['issuer'];
    }

    // Algorithm (default SHA1)
    Algorithm algorithm = Algorithm.sha1;
    if (params['algorithm'] != null) {
      switch (params['algorithm']!.toUpperCase()) {
        case 'SHA1':
          algorithm = Algorithm.sha1;
          break;
        case 'SHA256':
          algorithm = Algorithm.sha256;
          break;
        case 'SHA512':
          algorithm = Algorithm.sha512;
          break;
        default:
          throw QrScanException(
              'Unsupported algorithm: ${params['algorithm']}');
      }
    }

    // Digits (default 6)
    int digits = 6;
    if (params['digits'] != null) {
      digits = int.tryParse(params['digits']!) ?? 6;
      if (digits < 6 || digits > 8) {
        throw QrScanException('Digits must be 6, 7, or 8');
      }
    }

    // Period for TOTP (default 30)
    int? period;
    if (type == OathType.totp && params['period'] != null) {
      period = int.tryParse(params['period']!);
    }

    // Counter for HOTP
    int? counter;
    if (type == OathType.hotp) {
      counter = int.tryParse(params['counter'] ?? '0') ?? 0;
    }

    return OtpAuthCredential(
      type: type,
      issuer: issuer,
      account: account,
      secret: secret,
      algorithm: algorithm,
      digits: digits,
      period: period,
      counter: counter,
    );
  }
}

/// Exception thrown when QR scanning fails.
class QrScanException implements Exception {
  final String message;

  QrScanException(this.message);

  @override
  String toString() => message;
}
