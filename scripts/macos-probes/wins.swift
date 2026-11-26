import CoreGraphics
let owner = CommandLine.arguments[1]
let l = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
for w in l where (w[kCGWindowOwnerName as String] as? String) == owner {
  let b = w[kCGWindowBounds as String] as! [String: Double]
  print(w[kCGWindowNumber as String] as! Int, w[kCGWindowLayer as String] as! Int, (w[kCGWindowName as String] as? String) ?? "", Int(b["X"]!), Int(b["Y"]!), Int(b["Width"]!), Int(b["Height"]!))
}
