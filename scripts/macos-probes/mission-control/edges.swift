import AppKit

let a = CommandLine.arguments
let img = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: a[1])))!
let p = img.bitmapData!, bpr = img.bytesPerRow, spp = img.samplesPerPixel
let n = a[2...6].map { Int($0)! }
let x0 = n[0], x1 = n[1], y0 = n[2], y1 = n[3], tol = n[4]
let targets = a.dropFirst(7).map { $0.split(separator: ",").map { Int($0)! } }
func near(_ x: Int, _ y: Int, _ t: [Int]) -> Bool {
  let o = y * bpr + x * spp
  return (0..<3).allSatisfy { abs(Int(p[o + $0]) - t[$0]) <= tol }
}
for y in y0...y1 {
  let firsts = targets.map { t in (stride(from: x0, through: x1, by: x1 >= x0 ? 1 : -1).first { near($0, y, t) }).map(String.init) ?? "-" }
  print(y, firsts.joined(separator: " "))
}
