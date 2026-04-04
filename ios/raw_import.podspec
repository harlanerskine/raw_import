Pod::Spec.new do |s|
  s.name             = 'raw_import'
  s.version          = '0.1.0'
  s.summary          = 'RAW image import plugin for Flutter via pure-Rust rawler.'
  s.description      = <<-DESC
  A Flutter plugin for decoding camera RAW image formats (DNG, CR3, NEF, ARW,
  RAF, ORF) to RGB pixels using the rawler Rust crate via FFI.
                       DESC
  s.homepage         = 'https://github.com/Township-Innovation/raw_import'
  s.license          = { :file => '../LICENSE' }
  s.author           = { 'Township Innovation' => 'info@township.co' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.dependency 'Flutter'
  s.platform         = :ios, '13.0'
  s.swift_version    = '5.0'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'OTHER_LDFLAGS' => '-force_load ${BUILT_PRODUCTS_DIR}/libraw_import.a',
  }

  s.script_phase = {
    :name => 'Build Rust library',
    :script => 'sh "$PODS_TARGET_SRCROOT/../cargokit/build_pod.sh" ../rust/',
    :execution_position => :before_compile,
    :input_files => ['${BUILT_PRODUCTS_DIR}/cargokit_phony'],
    :output_files => ['${BUILT_PRODUCTS_DIR}/libraw_import.a'],
  }
end
