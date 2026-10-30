// Independent macOS decoder for the fictional Flutter QR fixture only.
import Foundation
import CoreImage

guard CommandLine.arguments.count == 2,
      let image = CIImage(contentsOf: URL(fileURLWithPath: CommandLine.arguments[1])),
      let detector = CIDetector(ofType: CIDetectorTypeQRCode, context: nil,
          options: [CIDetectorAccuracy: CIDetectorAccuracyHigh]) else {
    fatalError("Fictional QR image or decoder unavailable")
}
let decoded = detector.features(in: image).compactMap { ($0 as? CIQRCodeFeature)?.messageString }
guard decoded == ["https://example.invalid/fictional-homeverb-qr"] else {
    fatalError("Fictional QR decode mismatch")
}
print("Independent Core Image decoder: fictional QR payload matched")
