import AppKit
let a = CommandLine.arguments
func load(_ p: String) -> NSBitmapImageRep { NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: p)))! }
let u = load(a[1]), v = load(a[2])
let n = a[3...6].map { Int($0)! }
var cnt = 0, mx = 0, rows = [Int: Int]()
for y in n[1]..<(n[1] + n[3]) {
  for x in n[0]..<(n[0] + n[2]) {
    let o = y * u.bytesPerRow + x * u.samplesPerPixel, q = y * v.bytesPerRow + x * v.samplesPerPixel
    var d = 0
    for c in 0..<3 { d = max(d, abs(Int(u.bitmapData![o + c]) - Int(v.bitmapData![q + c]))) }
    if d > 0 { cnt += 1; mx = max(mx, d); rows[y, default: 0] += 1 }
  }
}
print("differing", cnt, "max", mx, "rows", rows.keys.min() ?? -1, rows.keys.max() ?? -1)
