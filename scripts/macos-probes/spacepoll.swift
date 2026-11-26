import AppKit
let key = Int(CommandLine.arguments[1])!
let id = CGMainDisplayID()
func sig() -> [Float] {
  let img = CGDisplayCreateImage(id)!
  let w = 64, h = 36
  var px = [UInt8](repeating: 0, count: w*h*4)
  let ctx = CGContext(data: &px, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w*4, space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
  ctx.interpolationQuality = .low
  ctx.draw(img, in: CGRect(x: 0, y: 0, width: w, height: h))
  return (0..<w*h).map { Float(px[$0*4]) + Float(px[$0*4+1]) + Float(px[$0*4+2]) }
}
var frames = [(Double, [Float])]()
let t0 = Date()
var fired = false
while Date().timeIntervalSince(t0) < 2.2 {
  let t = Date().timeIntervalSince(t0)
  if !fired && t > 0.4 { fired = true
    let p = Process(); p.launchPath = "/usr/bin/osascript"
    p.arguments = ["-e", "tell application \"System Events\" to key code \(key) using control down"]; try! p.run() }
  frames.append((t, sig()))
}
let base = frames[0].1, end = frames.last!.1
func d(_ a: [Float], _ b: [Float]) -> Float { zip(a, b).map { abs($0 - $1) }.reduce(0, +) / Float(a.count) }
let total = d(base, end)
print("samples", frames.count, "span", frames.last!.0, "total-diff", total)
var prev = base
for (t, s) in frames { let fromBase = d(base, s) / max(total, 1); let step = d(prev, s)
  if step > 0.05 || (fromBase > 0.001 && fromBase < 0.999) { print(String(format: "%.3f progress %.3f step %.2f", t, fromBase, step)) }
  prev = s }
