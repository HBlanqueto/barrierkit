import AppKit

let a = CommandLine.arguments
let row = Int(a[1])!
for f in a.dropFirst(2) {
  let img = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: f)))!
  let p = img.bitmapData!, bpr = img.bytesPerRow, spp = img.samplesPerPixel
  var best = (0, -1), start = -1
  for x in 0...img.pixelsWide {
    let o = row * bpr + x * spp
    let black = x < img.pixelsWide && Int(p[o]) + Int(p[o + 1]) + Int(p[o + 2]) < 12
    if black && start < 0 { start = x }
    if !black && start >= 0 { if x - start > best.1 - best.0 + 1 { best = (start, x - 1) }; start = -1 }
  }
  print(URL(fileURLWithPath: f).lastPathComponent, best.0, best.1)
}
