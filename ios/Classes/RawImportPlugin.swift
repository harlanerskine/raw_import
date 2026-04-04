import Flutter
import UIKit

public class RawImportPlugin: NSObject, FlutterPlugin {
  public static func register(with registrar: FlutterPluginRegistrar) {
    // FFI plugin — no method channels needed.
    // The Rust library is loaded automatically via the static lib linkage.
  }
}
