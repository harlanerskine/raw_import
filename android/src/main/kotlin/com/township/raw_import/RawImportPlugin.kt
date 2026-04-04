package com.township.raw_import

import io.flutter.embedding.engine.plugins.FlutterPlugin

class RawImportPlugin : FlutterPlugin {
    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        // FFI plugin — no method channels needed.
        // The Rust shared library is loaded automatically via JNI.
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {}
}
