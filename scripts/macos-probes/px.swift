import AppKit
let a = CommandLine.arguments
let img = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: a[1])))!
let x = Int(a[2])!, y0 = Int(a[3])!, y1 = Int(a[4])!
var prev = ""
for y in y0...y1 { let c = img.colorAt(x: x, y: y)!.usingColorSpace(.sRGB)!
  let s = "\(Int(c.redComponent*255)),\(Int(c.greenComponent*255)),\(Int(c.blueComponent*255))"
  if s != prev { print(y, s); prev = s } }
