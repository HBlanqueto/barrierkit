import AppKit
let a = CommandLine.arguments
func load(_ p: String) -> NSBitmapImageRep { NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: p)))! }
let bare = load(a[1]), over = load(a[2])
let n = a[3...6].map { Int($0)! }
let byRow = a.count > 7 && a[7] == "row"
let outer = byRow ? n[1]..<(n[1] + n[3]) : n[0]..<(n[0] + n[2])
let inner = byRow ? n[0]..<(n[0] + n[2]) : n[1]..<(n[1] + n[3])
for i in outer {
  var s = 0.0, k = 0.0
  for j in inner {
    let x = byRow ? j : i, y = byRow ? i : j
    let o = y * bare.bytesPerRow + x * bare.samplesPerPixel, q = y * over.bytesPerRow + x * over.samplesPerPixel
    for c in 0..<3 { let b = Double(bare.bitmapData![o + c]); if b > 20 { s += Double(over.bitmapData![q + c]) / b; k += 1 } }
  }
  print(i, String(format: "%.4f", k > 0 ? 1 - s / k : .nan))
}
