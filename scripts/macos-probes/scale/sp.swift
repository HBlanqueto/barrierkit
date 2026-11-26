import AppKit

// sp: backdrop (level 3) plus probe scenes (level 4/5), driven by the file `mode`, positions in device px converted by the scale.
// backdrop: gray:v | edgev:xpx:a:b | line:xpx:a:b (1 device px) ; scene:none|dark|aqua|noshadow|pop|vev ; off ; quit
// writes `ack` and `meta` (scale, screen, every probe window frame in device px, top-left origin, key state)
let dir = ProcessInfo.processInfo.environment["PROBE_DIR"]!
let screen = NSScreen.main!
let cs = screen.colorSpace!
let S = screen.backingScaleFactor
let W = screen.frame.width, H = screen.frame.height

func dev(_ v: Int) -> CGColor { NSColor(colorSpace: cs, components: [CGFloat(v) / 255, CGFloat(v) / 255, CGFloat(v) / 255, 1], count: 4).cgColor }

final class Pane: NSView {
  var mode = "gray:128"
  override var isFlipped: Bool { true }
  override func draw(_ dirty: NSRect) {
    let c = NSGraphicsContext.current!.cgContext
    let p = mode.split(separator: ":").map(String.init)
    func fill(_ v: String, _ r: CGRect) { c.setFillColor(dev(Int(v)!)); c.fill(r) }
    switch p[0] {
    case "gray": fill(p[1], bounds)
    case "edgev": let x = CGFloat(Int(p[1])!) / S; fill(p[2], CGRect(x: 0, y: 0, width: x, height: H)); fill(p[3], CGRect(x: x, y: 0, width: W - x, height: H))
    case "line": fill(p[2], bounds); fill(p[3], CGRect(x: CGFloat(Int(p[1])!) / S, y: 0, width: 1 / S, height: H))
    default: fill("128", bounds)
    }
  }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let bd = NSWindow(contentRect: screen.frame, styleMask: .borderless, backing: .buffered, defer: false)
bd.level = NSWindow.Level(rawValue: 3)
bd.hasShadow = false
bd.ignoresMouseEvents = true
bd.isReleasedWhenClosed = false
bd.collectionBehavior = [.canJoinAllSpaces, .stationary]
let pane = Pane(frame: NSRect(origin: .zero, size: screen.frame.size))
bd.contentView = pane

let slotA = NSPoint(x: 80, y: 100), slotB = NSPoint(x: 560, y: 100), size = NSSize(width: 300, height: 200)
var wins: [(String, NSWindow)] = []
var pop: NSPopover?

func titled(_ dark: Bool, _ at: NSPoint, _ sz: NSSize = size, transparent: Bool = false) -> NSWindow {
  let w = NSWindow(contentRect: NSRect(origin: .zero, size: sz), styleMask: transparent ? [.titled, .closable, .fullSizeContentView] : [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
  w.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
  w.title = "Probe"
  if transparent { w.titlebarAppearsTransparent = true; w.title = "" }
  w.level = NSWindow.Level(rawValue: 4)
  w.isReleasedWhenClosed = false
  w.setFrame(NSRect(x: at.x, y: H - at.y - sz.height, width: sz.width, height: sz.height), display: true)
  return w
}
func vev(_ m: NSVisualEffectView.Material, _ at: NSPoint) -> NSWindow {
  let w = titled(true, at, transparent: true)
  let v = NSVisualEffectView(); v.material = m; v.blendingMode = .behindWindow; v.state = .active
  w.contentView = v
  return w
}
func clear() { pop?.close(); pop = nil; for (_, w) in wins { w.orderOut(nil) }; wins = [] }
func scene(_ s: String) {
  clear()
  switch s {
  case "dark", "aqua":
    let d = titled(true, slotA), a = titled(false, slotB)
    wins = [("titled-dark", d), ("titled-aqua", a)]
  case "noshadow":
    let d = titled(true, slotA), a = titled(false, slotB); d.hasShadow = false; a.hasShadow = false
    wins = [("noshadow-dark", d), ("noshadow-aqua", a)]
  case "vev": wins = [("vev-sidebar", vev(.sidebar, slotA)), ("vev-windowBackground", vev(.windowBackground, slotB))]
  case "pop":
    let anchor = titled(true, slotB, NSSize(width: 300, height: 60))
    wins = [("anchor", anchor)]
  default: break
  }
  NSApp.activate(ignoringOtherApps: true)
  for (_, w) in wins { w.orderFrontRegardless() }
  if s == "aqua" { wins[1].1.makeKeyAndOrderFront(nil) } else if let f = wins.first { f.1.makeKeyAndOrderFront(nil) }
  if s == "pop" {
    let p = NSPopover(); let vc = NSViewController(); vc.view = NSView(frame: NSRect(x: 0, y: 0, width: 240, height: 160))
    p.contentViewController = vc; p.behavior = .applicationDefined; p.appearance = NSAppearance(named: .darkAqua)
    p.show(relativeTo: NSRect(x: 140, y: 10, width: 20, height: 20), of: wins[0].1.contentView!, preferredEdge: .minY)
    vc.view.window?.level = NSWindow.Level(rawValue: 5)
    pop = p
    wins.append(("popover", vc.view.window!))
  }
}
func meta() {
  var m = "scale \(S) screen \(Int(W)) \(Int(H)) visible \(Int(screen.visibleFrame.minX)) \(Int(H - screen.visibleFrame.maxY)) \(Int(screen.visibleFrame.width)) \(Int(screen.visibleFrame.height))\n"
  for (n, w) in wins { let f = w.frame; m += "\(n) \(Int(f.minX * S)) \(Int((H - f.maxY) * S)) \(Int(f.width * S)) \(Int(f.height * S)) key=\(w.isKeyWindow)\n" }
  try? m.write(toFile: "\(dir)/meta", atomically: true, encoding: .utf8)
}

var last = ""
func apply(_ m: String) {
  let p = m.split(separator: ":").map(String.init)
  switch p.first ?? "" {
  case "off": bd.orderOut(nil)
  case "quit": clear(); exit(0)
  case "scene": scene(p[1])
  default: pane.mode = m; pane.needsDisplay = true; pane.display(); bd.orderFrontRegardless()
  }
}
Timer.scheduledTimer(withTimeInterval: 0.02, repeats: true) { _ in
  guard let m = try? String(contentsOfFile: "\(dir)/mode", encoding: .utf8) else { return }
  let t = m.trimmingCharacters(in: .whitespacesAndNewlines)
  if t == last { return }
  last = t
  apply(t)
  DispatchQueue.main.asyncAfter(deadline: .now() + (t.hasPrefix("scene") ? 0.8 : 0.15)) { meta(); try? t.write(toFile: "\(dir)/ack", atomically: true, encoding: .utf8) }
}
try? "off".write(toFile: "\(dir)/mode", atomically: true, encoding: .utf8)
app.run()
