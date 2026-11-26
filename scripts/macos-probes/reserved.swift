import AppKit
for s in NSScreen.screens { print("scale", s.backingScaleFactor, "frame", s.frame, "visible", s.visibleFrame) }
let l = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
for w in l {
  let o = w[kCGWindowOwnerName as String] as? String ?? ""
  let n = w[kCGWindowName as String] as? String ?? ""
  if o == "Dock" || n == "Menubar" {
    let b = w[kCGWindowBounds as String] as! [String: Double]
    print(w[kCGWindowLayer as String] as! Int, o, "|", n, "|", Int(b["X"]!), Int(b["Y"]!), Int(b["Width"]!), Int(b["Height"]!))
  }
}
