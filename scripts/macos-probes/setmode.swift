import AppKit
let a = CommandLine.arguments.dropFirst().map { Int($0)! }
let id = CGMainDisplayID()
let opts = [kCGDisplayShowDuplicateLowResolutionModes: true] as CFDictionary
let m = (CGDisplayCopyAllDisplayModes(id, opts) as! [CGDisplayMode]).first { $0.width == a[0] && $0.height == a[1] && $0.pixelWidth == a[2] && $0.refreshRate == 180 && $0.isUsableForDesktopGUI() }!
var cfg: CGDisplayConfigRef?
CGBeginDisplayConfiguration(&cfg)
CGConfigureDisplayWithDisplayMode(cfg, id, m, nil)
print(CGCompleteDisplayConfiguration(cfg, .forSession).rawValue)
