import AppKit

let a = CommandLine.arguments
let img = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: a[1])))!
let p = img.bitmapData!, bpr = img.bytesPerRow, spp = img.samplesPerPixel
let vertical = a[2] == "col"
let fixed = Int(a[3])!, from = Int(a[4])!, to = Int(a[5])!
var prev = "", start = from
func px(_ i: Int) -> String {
  let o = vertical ? i * bpr + fixed * spp : fixed * bpr + i * spp
  return "\(p[o]),\(p[o + 1]),\(p[o + 2])"
}
for i in from...to {
  let s = px(i)
  if s != prev { if !prev.isEmpty { print(start, i - 1, prev) }; prev = s; start = i }
}
print(start, to, prev)
