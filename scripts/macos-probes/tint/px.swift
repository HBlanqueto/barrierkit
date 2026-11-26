import AppKit
// px file.png x y [x y ...]: mean of 41x41 patch at each centre (raw bytes)
let a = CommandLine.arguments
let r = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: a[1])))!.converting(to: .sRGB, renderingIntent: .default)!
let spp = r.samplesPerPixel, bpr = r.bytesPerRow, p = r.bitmapData!
var i = 2
while i + 1 < a.count {
  let cx = Int(a[i])!, cy = Int(a[i+1])!; var s = [0.0, 0.0, 0.0]; var n = 0.0
  for y in cy-20...cy+20 { for x in cx-20...cx+20 { let o = y*bpr + x*spp; for c in 0..<3 { s[c] += Double(p[o+c]) }; n += 1 } }
  print(cx, cy, String(format: "%.1f %.1f %.1f", s[0]/n, s[1]/n, s[2]/n)); i += 2
}
