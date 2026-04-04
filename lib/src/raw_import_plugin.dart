import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:raw_import/src/rust/api/decoder.dart' as rust;
import 'package:raw_import/src/rust/frb_generated.dart';

/// Metadata and pixel data from a decoded RAW image.
class RawDecodeResult {
  /// Decoded pixel data as 8-bit sRGB (RGB, 3 bytes per pixel).
  final Uint8List pixels;

  /// Image width after demosaic and orientation.
  final int width;

  /// Image height after demosaic and orientation.
  final int height;

  /// Camera manufacturer.
  final String make;

  /// Camera model.
  final String model;

  /// ISO sensitivity.
  final int iso;

  const RawDecodeResult({
    required this.pixels,
    required this.width,
    required this.height,
    required this.make,
    required this.model,
    required this.iso,
  });
}

/// Metadata probed from a RAW file without full decode.
class RawImageInfo {
  /// Raw sensor width in pixels.
  final int width;

  /// Raw sensor height in pixels.
  final int height;

  /// Camera manufacturer.
  final String make;

  /// Camera model.
  final String model;

  /// ISO sensitivity.
  final int iso;

  /// Whether full decode is supported.
  final bool supported;

  /// Detected format name (e.g. "DNG", "CR3", "NEF").
  final String format;

  const RawImageInfo({
    required this.width,
    required this.height,
    required this.make,
    required this.model,
    required this.iso,
    required this.supported,
    required this.format,
  });
}

/// Exception thrown when RAW decoding fails.
class RawImportException implements Exception {
  final String message;
  const RawImportException(this.message);

  @override
  String toString() => 'RawImportException: $message';
}

/// Main entry point for the RAW import plugin.
///
/// Provides static methods for probing, decoding, and extracting previews
/// from camera RAW files. The Rust library must be initialised once via
/// [init] before any other calls.
class RawImport {
  RawImport._();

  static bool _initialized = false;

  /// Initialise the Rust FFI library. Must be called once at app startup.
  static Future<void> init() async {
    if (_initialized) return;
    await RustLib.init();
    _initialized = true;
  }

  /// Check if the given bytes represent a supported RAW format.
  ///
  /// This is a quick check based on magic bytes — it does not perform
  /// a full decode. Returns `false` for corrupt or unsupported files.
  static bool isSupported(Uint8List fileBytes) {
    return rust.isSupportedRaw(fileBytes: fileBytes);
  }

  /// Probe a RAW file for metadata without full decode.
  ///
  /// Returns sensor dimensions, camera make/model, ISO, and format.
  /// Fast enough for gallery listing.
  ///
  /// Throws [RawImportException] if the file is not a supported RAW format.
  static RawImageInfo probe(Uint8List fileBytes) {
    if (!isSupported(fileBytes)) {
      throw RawImportException('Not a supported RAW format');
    }
    try {
      final info = rust.probeRaw(fileBytes: fileBytes);
      return RawImageInfo(
        width: info.width,
        height: info.height,
        make: info.make,
        model: info.model,
        iso: info.iso,
        supported: info.supported,
        format: info.format,
      );
    } catch (e) {
      throw RawImportException(e.toString());
    }
  }

  /// Fully decode a RAW file to 8-bit sRGB pixels.
  ///
  /// Performs Bayer demosaic, white balance, color space conversion, and
  /// gamma correction. Returns packed RGB bytes (3 bytes per pixel).
  ///
  /// To create a `ui.Image` from the result:
  /// ```dart
  /// final result = RawImport.decode(bytes);
  /// final image = await RawImport.decodeToUiImage(bytes);
  /// ```
  ///
  /// Throws [RawImportException] if decode fails.
  static RawDecodeResult decode(Uint8List fileBytes) {
    if (!isSupported(fileBytes)) {
      throw RawImportException('Not a supported RAW format');
    }
    try {
      final result = rust.decodeRaw(fileBytes: fileBytes);
      return RawDecodeResult(
        pixels: Uint8List.fromList(result.pixels),
        width: result.width,
        height: result.height,
        make: result.make,
        model: result.model,
        iso: result.iso,
      );
    } catch (e) {
      throw RawImportException(e.toString());
    }
  }

  /// Decode a RAW file and return JPEG bytes.
  ///
  /// This is the fastest path into Flutter's image pipeline — the returned
  /// bytes can be passed directly to `ui.instantiateImageCodec`.
  ///
  /// Throws [RawImportException] if decode fails.
  static Uint8List decodeToJpeg(Uint8List fileBytes, {int quality = 92}) {
    if (!isSupported(fileBytes)) {
      throw RawImportException('Not a supported RAW format');
    }
    try {
      final bytes =
          rust.decodeRawToJpeg(fileBytes: fileBytes, quality: quality);
      return Uint8List.fromList(bytes);
    } catch (e) {
      throw RawImportException(e.toString());
    }
  }

  /// Decode a RAW file directly to a Flutter `ui.Image`.
  ///
  /// Convenience method that decodes to JPEG internally, then uses
  /// Flutter's codec to create a `ui.Image`.
  ///
  /// Throws [RawImportException] if decode fails.
  static Future<ui.Image> decodeToUiImage(Uint8List fileBytes,
      {int quality = 95}) async {
    final jpegBytes = decodeToJpeg(fileBytes, quality: quality);
    final codec = await ui.instantiateImageCodec(jpegBytes);
    final frame = await codec.getNextFrame();
    return frame.image;
  }

  /// Extract the embedded JPEG preview from a RAW file.
  ///
  /// Most cameras embed a full-size JPEG preview in the RAW file.
  /// Returns `null` if no preview is available.
  ///
  /// Throws [RawImportException] if the file cannot be read.
  static Uint8List? extractPreview(Uint8List fileBytes) {
    if (!isSupported(fileBytes)) {
      throw RawImportException('Not a supported RAW format');
    }
    try {
      final preview = rust.extractPreview(fileBytes: fileBytes);
      if (preview == null) return null;
      return Uint8List.fromList(preview);
    } catch (e) {
      throw RawImportException(e.toString());
    }
  }

  /// Supported RAW file extensions.
  static const Set<String> supportedExtensions = {
    'dng',
    'cr2',
    'cr3',
    'nef',
    'nrw',
    'arw',
    'srf',
    'sr2',
    'raf',
    'orf',
    'rw2',
    'pef',
    'srw',
    'erf',
    'kdc',
    'dcr',
    '3fr',
    'mrw',
  };

  /// Check if a file extension indicates a RAW format.
  static bool isSupportedExtension(String path) {
    final ext = path.split('.').last.toLowerCase();
    return supportedExtensions.contains(ext);
  }
}
