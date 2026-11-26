import AppKit

let a = CommandLine.arguments
let y0 = Int(a[1])!, y1 = Int(a[2])!
for f in a.dropFirst(3) {
  let img = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: f)))!
  let p = img.bitmapData!, bpr = img.bytesPerRow, spp = img.samplesPerPixel
  var edge = -1, red = 0
  for y in y0...y1 {
    for x in 0..<(img.pixelsWide * 63 / 64) {
      let o = y * bpr + x * spp
      let r = Int(p[o]), g = Int(p[o + 1]), b = Int(p[o + 2])
      if r > 170 && g < 80 && b < 80 { red += 1; continue }
      if r + g + b > 30 { edge = max(edge, x) }
    }
  }
  print(URL(fileURLWithPath: f).lastPathComponent, edge, red)
}
