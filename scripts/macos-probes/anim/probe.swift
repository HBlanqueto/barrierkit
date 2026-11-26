import AppKit

let dir = ProcessInfo.processInfo.environment["PROBE_DIR"]! + "/sig"
let args = CommandLine.arguments
let mode = args.count > 1 ? args[1] : "tile"
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let screen = NSScreen.main!
let H = screen.frame.height
var wins: [NSWindow] = []

func rgb(_ r: CGFloat, _ g: CGFloat, _ b: CGFloat) -> NSColor { NSColor(colorSpace: screen.colorSpace!, components: [r, g, b, 1], count: 4) }
func pump(_ s: Double) { RunLoop.current.run(until: Date().addingTimeInterval(s)) }
func waitFor(_ name: String) {
  let p = "\(dir)/\(name)"
  while !FileManager.default.fileExists(atPath: p) { pump(0.01) }
  try? FileManager.default.removeItem(atPath: p)
}
func signal(_ name: String) { FileManager.default.createFile(atPath: "\(dir)/\(name)", contents: nil) }
func flat(_ c: NSColor, _ x: CGFloat, _ yTop: CGFloat, _ w: CGFloat, _ h: CGFloat, titled: Bool = false) -> NSWindow {
  let style: NSWindow.StyleMask = titled ? [.titled, .closable, .miniaturizable, .resizable] : .borderless
  let win = NSWindow(contentRect: NSRect(x: x, y: H - yTop - h, width: w, height: h), styleMask: style, backing: .buffered, defer: false)
  win.backgroundColor = c
  win.isReleasedWhenClosed = false
  win.hasShadow = true
  if titled { win.setFrame(NSRect(x: x, y: H - yTop - h, width: w, height: h), display: false) }
  wins.append(win)
  return win
}
func iconImage(_ c: NSColor) -> NSImage {
  NSImage(size: NSSize(width: 512, height: 512), flipped: false) { r in c.setFill(); r.fill(); return true }
}
let tileColors: [String: NSColor] = ["red": rgb(1, 0, 0), "green": rgb(0, 1, 0), "blue": rgb(0, 0, 1), "yellow": rgb(1, 1, 0), "cyan": rgb(0, 1, 1), "magenta": rgb(1, 0, 1)]

func body() {
  switch mode {
  case "spaces":
    let red = flat(rgb(1, 0, 0), 0, 0, screen.frame.width, H)
    app.activate(ignoringOtherApps: true); red.makeKeyAndOrderFront(nil)
    signal("ready")
    waitFor("go2")
    let blue = flat(rgb(0, 0, 1), 0, 0, screen.frame.width, H)
    blue.orderFrontRegardless()
    signal("ready2")
  case "mc":
    flat(rgb(0, 1, 0), 300, 250, 900, 600).orderFrontRegardless()
    flat(rgb(1, 0, 1), 1360, 600, 900, 600).orderFrontRegardless()
    app.activate(ignoringOtherApps: true)
    signal("ready")
  case "min":
    let w = flat(rgb(0, 1, 0), 880, 300, 800, 600, titled: true)
    app.activate(ignoringOtherApps: true); w.makeKeyAndOrderFront(nil)
    signal("ready")
    while true {
      waitFor("mini"); w.miniaturize(nil)
      waitFor("demini"); w.deminiaturize(nil)
    }
  case "open":
    app.activate(ignoringOtherApps: true)
    signal("ready")
    let w = flat(rgb(0, 1, 0), 380, 200, 1800, 1000, titled: true)
    while true {
      waitFor("open")
      w.orderFront(nil)
      waitFor("close")
      w.orderOut(nil)
    }
  default:
    app.applicationIconImage = iconImage(tileColors[mode] ?? .gray)
    signal("ready-\(mode)")
  }
}

DispatchQueue.main.async {
  body()
  waitFor("quit-\(mode)")
  exit(0)
}
app.run()
