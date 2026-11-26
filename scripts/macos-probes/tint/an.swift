import AppKit
// an tag...: reads data/<tag>.meta and pngs, prints tag ap state label R G B
let dir = ProcessInfo.processInfo.environment["PROBE_DIR"]! + "/data"
func load(_ f: String) -> NSBitmapImageRep? { guard let d = try? Data(contentsOf: URL(fileURLWithPath: f)) else { return nil }; return NSBitmapImageRep(data: d)!.converting(to: .sRGB, renderingIntent: .default) }
func patch(_ r: NSBitmapImageRep, _ cx: Int, _ cy: Int) -> [Double] {
  let spp = r.samplesPerPixel, bpr = r.bytesPerRow, p = r.bitmapData!; var s = [0.0, 0.0, 0.0]
  for y in cy-20...cy+20 { for x in cx-20...cx+20 { let o = y*bpr + x*spp; for c in 0..<3 { s[c] += Double(p[o+c]) } } }
  return s.map { $0 / 1681 }
}
func pts(_ label: String) -> [(String, Int, Int)] {
  switch label {
  case "split": return [("split-sidebar", 100, 150), ("split-content", 360, 150)]
  case "glass": return [("glass", 260, 140)]
  default: return [(label, 130, 150), (label, 260, 150), (label, 390, 150)]
  }
}
for tag in CommandLine.arguments.dropFirst() {
  let lines = (try! String(contentsOfFile: "\(dir)/\(tag).meta", encoding: .utf8)).split(separator: "\n").map { $0.split(separator: " ").map(String.init) }.filter { $0.count == 7 }
  for ap in ["aqua", "dark"] {
    let wins = lines.filter { $0[1] == ap }
    var cache: [String: NSBitmapImageRep] = [:]
    for w in wins {
      let x = Int(w[3])!, y = Int(w[4])!
      for (state, file) in [("active", "\(tag)-\(ap)-key-\(w[2])"), ("inactive", "\(tag)-\(ap)-inactive")] {
        guard let r = cache[file] ?? load("\(dir)/\(file).png") else { continue }
        cache[file] = r
        var acc: [String: [Double]] = [:]; var n: [String: Double] = [:]
        for (l, px, py) in pts(w[2]) { let v = patch(r, x + px, y + py); acc[l] = zip(acc[l] ?? [0,0,0], v).map { $0 + $1 }; n[l, default: 0] += 1 }
        for (l, v) in acc.sorted(by: { $0.key < $1.key }) { print(tag, ap, state, l, v.map { String(format: "%.1f", $0 / n[l]!) }.joined(separator: " ")) }
      }
    }
  }
}
