import Foundation
import CoreGraphics
let sig = CommandLine.arguments[1]
func probeWin() -> String? {
  let l = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
  for w in l where (w[kCGWindowOwnerName as String] as? String) == "AnimProbe" {
    let b = w[kCGWindowBounds as String] as! [String: Any]
    if (b["Width"] as! Double) > 500 { return "\(b["X"]!) \(b["Y"]!) \(b["Width"]!) \(b["Height"]!) a=\(w[kCGWindowAlpha as String]!)" }
  }
  return nil
}
for _ in 0..<3 {
  FileManager.default.createFile(atPath: sig + "/open", contents: nil)
  let t0 = Date(); var last: String? = "x"
  while Date().timeIntervalSince(t0) < 0.4 {
    let s = probeWin(); if s != last { print(String(format: "%.1f", Date().timeIntervalSince(t0) * 1000), s ?? "none"); last = s }
    usleep(1000)
  }
  FileManager.default.createFile(atPath: sig + "/close", contents: nil); usleep(800000); print("--")
}
