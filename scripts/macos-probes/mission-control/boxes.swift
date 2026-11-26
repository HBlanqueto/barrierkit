import AppKit

let a = CommandLine.arguments
let img = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: a[1])))!
let spp = img.samplesPerPixel, bpr = img.bytesPerRow, p = img.bitmapData!
let W = img.pixelsWide, H = img.pixelsHigh
let tol = Int(a[2])!
let targets = a.dropFirst(3).map { $0.split(separator: ",").map { Int($0)! } }
var mn = targets.map { _ in [Int.max, Int.max] }, mx = targets.map { _ in [-1, -1] }, ct = targets.map { _ in 0 }
let minY = Int(ProcessInfo.processInfo.environment["MINY"] ?? "0")!
for y in minY..<H {
  for x in 0..<W {
    let o = y * bpr + x * spp
    let r = Int(p[o]), g = Int(p[o + 1]), b = Int(p[o + 2])
    for (i, t) in targets.enumerated() where abs(r - t[0]) <= tol && abs(g - t[1]) <= tol && abs(b - t[2]) <= tol {
      mn[i][0] = min(mn[i][0], x); mn[i][1] = min(mn[i][1], y); mx[i][0] = max(mx[i][0], x); mx[i][1] = max(mx[i][1], y); ct[i] += 1
    }
  }
}
for i in targets.indices {
  if ct[i] == 0 { print(i, "none"); continue }
  print(i, mn[i][0], mn[i][1], mx[i][0] - mn[i][0] + 1, mx[i][1] - mn[i][1] + 1, ct[i])
}
