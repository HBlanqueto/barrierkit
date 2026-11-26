import AppKit

let a = CommandLine.arguments
let cols = Int(a[2])!
let files = Array(a.dropFirst(3))
let imgs = files.map { NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: $0)))!.cgImage! }
let w = imgs[0].width, h = imgs[0].height, label = 18
let rows = (imgs.count + cols - 1) / cols
let ctx = CGContext(data: nil, width: cols * (w + 4), height: rows * (h + label + 4), bitsPerComponent: 8, bytesPerRow: 0, space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
ctx.setFillColor(gray: 1, alpha: 1)
ctx.fill(CGRect(x: 0, y: 0, width: ctx.width, height: ctx.height))
NSGraphicsContext.current = NSGraphicsContext(cgContext: ctx, flipped: false)
for (i, img) in imgs.enumerated() {
  let x = (i % cols) * (w + 4), y = ctx.height - (i / cols + 1) * (h + label + 4)
  ctx.draw(img, in: CGRect(x: x, y: y, width: w, height: h))
  let name = URL(fileURLWithPath: files[i]).deletingPathExtension().lastPathComponent
  NSString(string: name).draw(at: NSPoint(x: x + 4, y: y + h + 2), withAttributes: [.font: NSFont.systemFont(ofSize: 14), .foregroundColor: NSColor.black])
}
try! NSBitmapImageRep(cgImage: ctx.makeImage()!).representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: a[1]))
