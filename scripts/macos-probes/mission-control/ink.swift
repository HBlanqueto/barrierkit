import AppKit
let a = CommandLine.arguments
let img = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: a[1])))!
let n = a[2...5].map { Int($0)! }
let bg = a[6].split(separator: ",").map { Int($0)! }, tol = Int(a[7])!
var cols = [Int](), rows = [Int]()
var colHit = [Bool](repeating: false, count: n[2]), rowHit = [Bool](repeating: false, count: n[3])
for y in 0..<n[3] { for x in 0..<n[2] {
  let o = (n[1] + y) * img.bytesPerRow + (n[0] + x) * img.samplesPerPixel
  var d = 0
  for c in 0..<3 { d = max(d, abs(Int(img.bitmapData![o + c]) - bg[c])) }
  if d > tol { colHit[x] = true; rowHit[y] = true }
} }
func runs(_ h: [Bool], _ off: Int) -> String {
  var s = [String](), start = -1
  for (i, v) in (h + [false]).enumerated() { if v && start < 0 { start = i }; if !v && start >= 0 { s.append("\(start + off)-\(i - 1 + off)"); start = -1 } }
  return s.joined(separator: " ")
}
print("x:", runs(colHit, n[0])); print("y:", runs(rowHit, n[1]))
