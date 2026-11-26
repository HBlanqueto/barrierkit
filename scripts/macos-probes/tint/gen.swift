import AppKit
let dir = CommandLine.arguments[1]
let W = 2560, H = 1440
func write(_ name: String, _ f: (Int) -> (UInt8, UInt8, UInt8)) {
  let r = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: W, pixelsHigh: H, bitsPerSample: 8, samplesPerPixel: 3, hasAlpha: false, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: W * 3, bitsPerPixel: 24)!
  let p = r.bitmapData!
  for x in 0..<W { let c = f(x); for y in 0..<H { let o = y * W * 3 + x * 3; p[o] = c.0; p[o+1] = c.1; p[o+2] = c.2 } }
  let s = r.retagging(with: .sRGB)!
  try! s.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: "\(dir)/\(name).png"))
}
let solids: [(String, (UInt8, UInt8, UInt8))] = [("red",(255,0,0)),("green",(0,255,0)),("blue",(0,0,255)),("white",(255,255,255)),("black",(0,0,0)),("gray",(128,128,128)),("orange",(255,128,0)),("navy",(40,80,160)),("ice",(200,220,255))]
for (n, c) in solids { write(n) { _ in c } }
write("halves") { $0 < 1280 ? (255,0,0) : (0,0,255) }
write("gradient") { x in let t = Double(x) / Double(W - 1); return (UInt8((255 * (1 - t)).rounded()), 0, UInt8((255 * t).rounded())) }
