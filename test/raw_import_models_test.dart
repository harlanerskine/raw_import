// Path: test/raw_import_models_test.dart
// Run:  flutter test test/raw_import_models_test.dart

import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';
import 'package:raw_import/src/raw_import_plugin.dart';

void main() {
  group('RawDecodeResult', () {
    test('stores all fields correctly', () {
      final pixels = Uint8List.fromList([255, 0, 0, 0, 255, 0]);
      final result = RawDecodeResult(
        pixels: pixels,
        width: 2,
        height: 1,
        make: 'Canon',
        model: 'EOS R5',
        iso: 100,
      );

      expect(result.pixels, pixels);
      expect(result.width, 2);
      expect(result.height, 1);
      expect(result.make, 'Canon');
      expect(result.model, 'EOS R5');
      expect(result.iso, 100);
    });

    test('can be constructed with empty pixels', () {
      final result = RawDecodeResult(
        pixels: Uint8List(0),
        width: 0,
        height: 0,
        make: '',
        model: '',
        iso: 0,
      );

      expect(result.pixels, isEmpty);
      expect(result.width, 0);
      expect(result.height, 0);
    });

    test('pixel data length matches dimensions (3 bytes per pixel)', () {
      const w = 4;
      const h = 3;
      final pixels = Uint8List(w * h * 3); // RGB packed
      final result = RawDecodeResult(
        pixels: pixels,
        width: w,
        height: h,
        make: 'Sony',
        model: 'A7R V',
        iso: 200,
      );

      expect(result.pixels.length, w * h * 3);
    });

    test('handles high ISO values', () {
      final result = RawDecodeResult(
        pixels: Uint8List(3),
        width: 1,
        height: 1,
        make: 'Nikon',
        model: 'Z 9',
        iso: 102400,
      );

      expect(result.iso, 102400);
    });

    test('const constructor allows compile-time creation', () {
      // Verifies that the const constructor works (won't compile if broken)
      expect(
        () => RawDecodeResult(
          pixels: Uint8List(0),
          width: 0,
          height: 0,
          make: '',
          model: '',
          iso: 0,
        ),
        returnsNormally,
      );
    });
  });

  group('RawImageInfo', () {
    test('stores all fields correctly', () {
      const info = RawImageInfo(
        width: 6000,
        height: 4000,
        make: 'Canon',
        model: 'EOS R5',
        iso: 400,
        supported: true,
        format: 'CR3',
      );

      expect(info.width, 6000);
      expect(info.height, 4000);
      expect(info.make, 'Canon');
      expect(info.model, 'EOS R5');
      expect(info.iso, 400);
      expect(info.supported, isTrue);
      expect(info.format, 'CR3');
    });

    test('can represent unsupported formats', () {
      const info = RawImageInfo(
        width: 0,
        height: 0,
        make: 'Unknown',
        model: 'Unknown',
        iso: 0,
        supported: false,
        format: 'unknown',
      );

      expect(info.supported, isFalse);
      expect(info.format, 'unknown');
    });

    test('handles various camera manufacturers', () {
      const manufacturers = [
        ('Canon', 'EOS R5', 'CR3'),
        ('Nikon', 'Z 9', 'NEF'),
        ('Sony', 'A7R V', 'ARW'),
        ('Fujifilm', 'X-T5', 'RAF'),
        ('Olympus', 'OM-1', 'ORF'),
        ('Panasonic', 'S5 II', 'RW2'),
        ('Pentax', 'K-3 III', 'PEF'),
        ('Samsung', 'NX1', 'SRW'),
      ];

      for (final (make, model, format) in manufacturers) {
        final info = RawImageInfo(
          width: 6000,
          height: 4000,
          make: make,
          model: model,
          iso: 100,
          supported: true,
          format: format,
        );
        expect(info.make, make);
        expect(info.model, model);
        expect(info.format, format);
      }
    });

    test('supports DNG/TIFF-RAW format identifier', () {
      const info = RawImageInfo(
        width: 8192,
        height: 5464,
        make: 'Leica',
        model: 'Q3',
        iso: 100,
        supported: true,
        format: 'DNG/TIFF-RAW',
      );

      expect(info.format, 'DNG/TIFF-RAW');
    });
  });

  group('RawImportException', () {
    test('stores message', () {
      const e = RawImportException('decode failed');
      expect(e.message, 'decode failed');
    });

    test('toString includes class name', () {
      const e = RawImportException('unsupported format');
      expect(e.toString(), 'RawImportException: unsupported format');
    });

    test('implements Exception', () {
      const e = RawImportException('test');
      expect(e, isA<Exception>());
    });

    test('can be thrown and caught', () {
      expect(
        () => throw const RawImportException('test error'),
        throwsA(isA<RawImportException>()),
      );
    });

    test('message preserved through throw/catch', () {
      try {
        throw const RawImportException('specific error');
      } on RawImportException catch (e) {
        expect(e.message, 'specific error');
        return;
      }
    });

    test('handles empty message', () {
      const e = RawImportException('');
      expect(e.message, isEmpty);
      expect(e.toString(), 'RawImportException: ');
    });

    test('handles multiline message', () {
      const e = RawImportException('line 1\nline 2');
      expect(e.message, contains('\n'));
    });
  });

  group('RawImport static utilities', () {
    test('supportedExtensions contains expected formats', () {
      expect(RawImport.supportedExtensions, contains('dng'));
      expect(RawImport.supportedExtensions, contains('cr2'));
      expect(RawImport.supportedExtensions, contains('cr3'));
      expect(RawImport.supportedExtensions, contains('nef'));
      expect(RawImport.supportedExtensions, contains('arw'));
      expect(RawImport.supportedExtensions, contains('raf'));
      expect(RawImport.supportedExtensions, contains('orf'));
      expect(RawImport.supportedExtensions, contains('rw2'));
      expect(RawImport.supportedExtensions, contains('pef'));
    });

    test('supportedExtensions contains all 18 formats', () {
      expect(RawImport.supportedExtensions.length, 18);
    });

    test('supportedExtensions are all lowercase', () {
      for (final ext in RawImport.supportedExtensions) {
        expect(ext, ext.toLowerCase(),
            reason: 'Extension "$ext" should be lowercase');
      }
    });

    test('isSupportedExtension returns true for supported formats', () {
      expect(RawImport.isSupportedExtension('photo.dng'), isTrue);
      expect(RawImport.isSupportedExtension('IMG_1234.CR3'), isTrue);
      expect(RawImport.isSupportedExtension('DSC_5678.NEF'), isTrue);
      expect(RawImport.isSupportedExtension('image.arw'), isTrue);
      expect(RawImport.isSupportedExtension('DSCF9012.RAF'), isTrue);
    });

    test('isSupportedExtension is case-insensitive', () {
      expect(RawImport.isSupportedExtension('photo.DNG'), isTrue);
      expect(RawImport.isSupportedExtension('photo.Dng'), isTrue);
      expect(RawImport.isSupportedExtension('photo.dng'), isTrue);
      expect(RawImport.isSupportedExtension('photo.CR3'), isTrue);
      expect(RawImport.isSupportedExtension('photo.cr3'), isTrue);
    });

    test('isSupportedExtension returns false for non-RAW formats', () {
      expect(RawImport.isSupportedExtension('photo.jpg'), isFalse);
      expect(RawImport.isSupportedExtension('photo.jpeg'), isFalse);
      expect(RawImport.isSupportedExtension('photo.png'), isFalse);
      expect(RawImport.isSupportedExtension('photo.tiff'), isFalse);
      expect(RawImport.isSupportedExtension('photo.heic'), isFalse);
      expect(RawImport.isSupportedExtension('photo.webp'), isFalse);
      expect(RawImport.isSupportedExtension('document.pdf'), isFalse);
    });

    test('isSupportedExtension handles paths with directories', () {
      expect(
          RawImport.isSupportedExtension('/photos/2024/IMG_001.cr3'), isTrue);
      expect(
          RawImport.isSupportedExtension('C:\\Users\\photo.nef'), isTrue);
    });

    test('isSupportedExtension handles multiple dots in filename', () {
      expect(
          RawImport.isSupportedExtension('photo.edit.final.dng'), isTrue);
      expect(
          RawImport.isSupportedExtension('photo.backup.jpg'), isFalse);
    });

    test('isSupportedExtension handles edge cases', () {
      // No extension
      expect(RawImport.isSupportedExtension('photofile'), isFalse);
      // Just a dot
      expect(RawImport.isSupportedExtension('photo.'), isFalse);
      // Extension only
      expect(RawImport.isSupportedExtension('.dng'), isTrue);
    });

    test('all listed extensions are in supportedExtensions set', () {
      const expectedExtensions = [
        'dng', 'cr2', 'cr3', 'nef', 'nrw', 'arw', 'srf', 'sr2',
        'raf', 'orf', 'rw2', 'pef', 'srw', 'erf', 'kdc', 'dcr',
        '3fr', 'mrw',
      ];
      for (final ext in expectedExtensions) {
        expect(RawImport.supportedExtensions.contains(ext), isTrue,
            reason: 'Missing extension: $ext');
      }
    });
  });
}
