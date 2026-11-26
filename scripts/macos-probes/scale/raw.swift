import AppKit
// raw dir: every *.png -> *.raw (8-bit raw device R channel, row-major) and dims printed once
let d = CommandLine.arguments[1]
for f in try! FileManager.default.contentsOfDirectory(atPath: d) where f.hasSuffix(".png") {
  let out = d + "/" + f.dropLast(4) + ".raw"
  if FileManager.default.fileExists(atPath: out) { continue }
  let r = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: d + "/" + f)))!
  let w = r.pixelsWide, h = r.pixelsHigh, spp = r.samplesPerPixel, bpr = r.bytesPerRow, p = r.bitmapData!
  var o = [UInt8](repeating: 0, count: w * h)
  for y in 0..<h { for x in 0..<w { o[y * w + x] = p[y * bpr + x * spp] } }
  FileManager.default.createFile(atPath: out, contents: Data(o))
  try! "\(w) \(h)".write(toFile: d + "/dims", atomically: true, encoding: .utf8)
}
