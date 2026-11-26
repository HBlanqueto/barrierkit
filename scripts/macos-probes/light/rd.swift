import AppKit

// rd file.png patch x y w h          -> mean and std per channel (raw device bytes) of the rect
// rd file.png row y x0 x1            -> raw R G B per pixel along a row
// rd file.png col x y0 y1            -> raw R G B per pixel along a column
// rd file.png srgb x y w h           -> mean per channel after conversion to sRGB
let a = CommandLine.arguments
let raw = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: a[1])))!
let r = a[2] == "srgb" ? raw.converting(to: .sRGB, renderingIntent: .default)! : raw
let spp = r.samplesPerPixel, bpr = r.bytesPerRow, p = r.bitmapData!
func px(_ x: Int, _ y: Int) -> [Int] { let o = y * bpr + x * spp; return [Int(p[o]), Int(p[o + 1]), Int(p[o + 2])] }
let n = a.dropFirst(3).map { Int($0)! }
switch a[2] {
case "patch", "srgb":
  var s = [0.0, 0.0, 0.0], q = [0.0, 0.0, 0.0], k = 0.0
  for y in n[1]..<(n[1] + n[3]) { for x in n[0]..<(n[0] + n[2]) { let v = px(x, y); for c in 0..<3 { s[c] += Double(v[c]); q[c] += Double(v[c] * v[c]) }; k += 1 } }
  let m = s.map { $0 / k }
  print(String(format: "%.1f %.1f %.1f  std %.2f %.2f %.2f", m[0], m[1], m[2], sqrt(max(0, q[0] / k - m[0] * m[0])), sqrt(max(0, q[1] / k - m[1] * m[1])), sqrt(max(0, q[2] / k - m[2] * m[2]))))
case "row": for x in n[1]...n[2] { let v = px(x, n[0]); print(x, v[0], v[1], v[2]) }
case "col": for y in n[1]...n[2] { let v = px(n[0], y); print(y, v[0], v[1], v[2]) }
case "cmp":
  var mx = 0, cnt = 0
  for y in n[0]...n[1] { for x in n[3]...n[4] { let u = px(x, y), v = px(x, y + n[2]); for c in 0..<3 { let d = abs(u[c] - v[c]); mx = max(mx, d); if d > 0 { cnt += 1 } } } }
  print("maxdiff", mx, "nonzero", cnt)
case "minmax":
  var lo = [255, 255, 255], hi = [0, 0, 0], lumLo = 999, lumHi = -1, pLo = [0, 0, 0], pHi = [0, 0, 0]
  for y in n[1]..<(n[1] + n[3]) { for x in n[0]..<(n[0] + n[2]) { let v = px(x, y); for c in 0..<3 { lo[c] = min(lo[c], v[c]); hi[c] = max(hi[c], v[c]) }; let l = v[0] + v[1] + v[2]; if l < lumLo { lumLo = l; pLo = v }; if l > lumHi { lumHi = l; pHi = v } } }
  print("darkest", pLo, "lightest", pHi)
default: break
}
