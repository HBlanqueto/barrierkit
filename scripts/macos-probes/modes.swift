import AppKit
let id = CGMainDisplayID()
let opts = [kCGDisplayShowDuplicateLowResolutionModes: true] as CFDictionary
var seen = Set<String>()
for d in CGDisplayCopyAllDisplayModes(id, opts) as! [CGDisplayMode] where d.isUsableForDesktopGUI() && d.refreshRate >= 179 {
  let s = "\(d.width)x\(d.height) px \(d.pixelWidth)x\(d.pixelHeight)"
  if seen.insert(s).inserted { print(s) }
}
