import CoreGraphics
let l = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
for w in l {
  let b = w[kCGWindowBounds as String] as! [String: Double]
  let o = w[kCGWindowOwnerName as String] as? String ?? ""
  let n = w[kCGWindowName as String] as? String ?? ""
  let layer = w[kCGWindowLayer as String] as! Int
  print(layer, o, "|", n, "|", Int(b["X"]!), Int(b["Y"]!), Int(b["Width"]!), Int(b["Height"]!))
}
