import AppKit

// bd: full-screen backdrop at level 3 driven by the file `mode`; writes `ack` once the mode is on screen.
// modes: off | gray:v | rgb:r,g,b | edgev:x:a:b | line:x:a:b | lvl:n | win:on | win:off | quit
let dir = ProcessInfo.processInfo.environment["PROBE_DIR"]!
let screen = NSScreen.main!
let cs = screen.colorSpace!
let W = screen.frame.width, H = screen.frame.height

func dev(_ r: Int, _ g: Int, _ b: Int) -> CGColor {
  NSColor(colorSpace: cs, components: [CGFloat(r) / 255, CGFloat(g) / 255, CGFloat(b) / 255, 1], count: 4).cgColor
}

final class Pane: NSView {
  var mode = "gray:128"
  override var isFlipped: Bool { true }
  override func draw(_ dirty: NSRect) {
    let c = NSGraphicsContext.current!.cgContext
    let p = mode.split(separator: ":").map(String.init)
    func fill(_ col: CGColor, _ r: CGRect) { c.setFillColor(col); c.fill(r) }
    func g(_ s: String) -> CGColor { let v = Int(s)!; return dev(v, v, v) }
    switch p[0] {
    case "gray": fill(g(p[1]), bounds)
    case "rgb": let v = p[1].split(separator: ",").map { Int($0)! }; fill(dev(v[0], v[1], v[2]), bounds)
    case "edgev":
      let x = CGFloat(Int(p[1])!)
      fill(g(p[2]), CGRect(x: 0, y: 0, width: x, height: H)); fill(g(p[3]), CGRect(x: x, y: 0, width: W - x, height: H))
    case "line":
      fill(g(p[2]), bounds); fill(g(p[3]), CGRect(x: CGFloat(Int(p[1])!), y: 0, width: 1, height: H))
    default: fill(g("128"), bounds)
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

final class Side: NSViewController { override func loadView() { view = NSView(frame: NSRect(x: 0, y: 0, width: 220, height: 450)) } }
final class Main: NSViewController {
  override func loadView() {
    let v = NSView(frame: NSRect(x: 0, y: 0, width: 480, height: 450))
    let b = NSButton(title: "Probe", target: nil, action: nil)
    b.bezelStyle = .glass
    b.frame = NSRect(x: 40, y: 40, width: 160, height: 40)
    v.addSubview(b)
    view = v
  }
}
var probeWin: NSWindow?
var pop: NSPopover?
func showWin() {
  let split = NSSplitViewController()
  split.addSplitViewItem(NSSplitViewItem(sidebarWithViewController: Side()))
  split.addSplitViewItem(NSSplitViewItem(viewController: Main()))
  let w = NSWindow(contentViewController: split)
  w.styleMask = [.titled, .closable, .resizable, .fullSizeContentView]
  w.title = "Probe"
  w.level = NSWindow.Level(rawValue: 4)
  w.isReleasedWhenClosed = false
  w.setFrame(NSRect(x: 100, y: H - 150 - 450, width: 700, height: 450), display: true)
  w.orderFrontRegardless()
  probeWin = w
  let p = NSPopover()
  let vc = NSViewController()
  vc.view = NSView(frame: NSRect(x: 0, y: 0, width: 400, height: 300))
  p.contentViewController = vc
  p.behavior = .applicationDefined
  let main = split.splitViewItems[1].viewController.view
  p.show(relativeTo: NSRect(x: 300, y: 380, width: 20, height: 20), of: main, preferredEdge: .maxX)
  vc.view.window?.level = NSWindow.Level(rawValue: 5)
  pop = p
}

var last = ""
func apply(_ m: String) {
  let p = m.split(separator: ":").map(String.init)
  switch p.first ?? "" {
  case "off": bd.orderOut(nil)
  case "quit": exit(0)
  case "lvl": bd.level = NSWindow.Level(rawValue: Int(p[1])!)
  case "win": if p[1] == "on" { showWin() } else { pop?.close(); probeWin?.orderOut(nil); probeWin = nil }
  default: pane.mode = m; pane.needsDisplay = true; pane.display(); bd.orderFrontRegardless()
  }
}
Timer.scheduledTimer(withTimeInterval: 0.02, repeats: true) { _ in
  guard let m = try? String(contentsOfFile: "\(dir)/mode", encoding: .utf8) else { return }
  let t = m.trimmingCharacters(in: .whitespacesAndNewlines)
  if t == last { return }
  last = t
  apply(t)
  DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) { try? t.write(toFile: "\(dir)/ack", atomically: true, encoding: .utf8) }
}
app.run()
