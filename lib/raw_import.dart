/// RAW image import plugin for Flutter.
///
/// Decodes camera RAW formats (DNG, CR3, NEF, ARW, RAF, ORF) to RGB pixels
/// using the rawler Rust crate via FFI.
///
/// ## Quick Start
///
/// ```dart
/// import 'package:raw_import/raw_import.dart';
///
/// // 1. Initialise (once)
/// await RawImport.init();
///
/// // 2. Check if a file is a supported RAW format
/// final bytes = await File('IMG_1234.DNG').readAsBytes();
/// if (RawImport.isSupported(bytes)) {
///   // 3a. Fast path: decode to JPEG (for instantiateImageCodec)
///   final jpeg = RawImport.decodeToJpeg(bytes, quality: 92);
///
///   // 3b. Full path: decode to raw RGB pixels
///   final result = RawImport.decode(bytes);
///   print('${result.width}x${result.height} from ${result.make} ${result.model}');
/// }
/// ```
library;

export 'src/raw_import_plugin.dart';
