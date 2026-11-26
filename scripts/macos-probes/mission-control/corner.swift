import AppKit
let a = CommandLine.arguments
let img = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: a[1])))!
let x0 = Int(a[2])!, yBottom = Int(a[3])!, ch = Int(a[4])!, full = Double(a[5])!, bg = Double(a[6])!
for k in 0..<14 {
  let y = yBottom - k
  var cover = 0.0
  for x in x0..<(x0 + 30) {
    let v = Double(img.bitmapData![y * img.bytesPerRow + x * img.samplesPerPixel + ch])
    cover += max(0, min(1, (v - bg) / (full - bg)))
  }
  print(k, String(format: "%.2f", 30 - cover))
}
