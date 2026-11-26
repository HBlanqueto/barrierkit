import AppKit
// sp tag ap label: for each window position prints x and R,G,B every 50 px across the window (row y=170)
let dir = ProcessInfo.processInfo.environment["PROBE_DIR"]! + "/data"
let a = CommandLine.arguments
let ch = Int(a[4]) ?? 0
let lines = (try! String(contentsOfFile: "\(dir)/\(a[1]).meta", encoding: .utf8)).split(separator: "\n").map { $0.split(separator: " ").map(String.init) }.filter { $0.count == 8 && $0[1] == a[2] && $0[2] == a[3] }
for w in lines {
  let r = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: "\(dir)/\(a[1])-\(a[2])-\(w[3]).png")))!.converting(to: .sRGB, renderingIntent: .default)!
  let x0 = Int(w[4])!, y0 = Int(w[5])!
  let spp = r.samplesPerPixel, bpr = r.bytesPerRow, p = r.bitmapData!
  var s = "\(w[3])"
  for x in stride(from: x0 + 25, to: x0 + 1000, by: 50) {
    var v = 0.0
    for y in y0+160...y0+180 { for xx in x-10...x+10 { v += Double(p[y*bpr + xx*spp + ch]) } }
    s += String(format: " %3.0f", v / 441)
  }
  print(s)
}
