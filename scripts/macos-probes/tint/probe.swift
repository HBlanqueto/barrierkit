import AppKit

let dir = "/private/tmp/claude-501/-Users-humbe--claude-spares-spare-wxcbfZ/815469d1-10d1-46d0-9155-bfb2393396d4/scratchpad/tint/data"
try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
let args = CommandLine.arguments
let mode = args[1]
let tag = args[2]
let useBackdrop = args.count > 3 && args[3] == "backdrop"
let screen = NSScreen.main!
let H = screen.frame.height
let level = NSWindow.Level(rawValue: 901)
var meta = ""

func pump(_ s: Double) { RunLoop.current.run(until: Date().addingTimeInterval(s)) }
func shoot(_ name: String) {
    pump(0.6)
    let png = "\(dir)/\(name).png"
    FileManager.default.createFile(atPath: "\(dir)/request", contents: Data(png.utf8))
    while FileManager.default.fileExists(atPath: "\(dir)/request") { pump(0.01) }
}
func tl(_ w: NSWindow) -> String { let f = w.frame; return "\(Int(f.minX)) \(Int(H - f.maxY)) \(Int(f.width)) \(Int(f.height))" }
func place(_ w: NSWindow, _ x: CGFloat, _ y: CGFloat) { w.setFrameTopLeftPoint(NSPoint(x: x, y: H - y)) }

func window(_ dark: Bool, _ size: NSSize) -> NSWindow {
    let w = NSWindow(contentRect: NSRect(origin: .zero, size: size), styleMask: [.titled, .closable, .resizable, .fullSizeContentView], backing: .buffered, defer: false)
    w.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
    w.titlebarAppearsTransparent = true
    w.title = ""
    w.level = level
    w.isReleasedWhenClosed = false
    return w
}

func lights(_ name: String, _ w: NSWindow) { meta += "lights \(name) \(tl(w)) key=\(w.isKeyWindow) active=\(NSApp.isActive)\n" }

let materials: [(String, NSVisualEffectView.Material)] = [
    ("windowBackground", .windowBackground), ("underWindowBackground", .underWindowBackground), ("sidebar", .sidebar),
    ("headerView", .headerView), ("contentBackground", .contentBackground), ("menu", .menu), ("popover", .popover), ("hudWindow", .hudWindow),
]

func vev(_ dark: Bool, _ m: NSVisualEffectView.Material, _ size: NSSize) -> NSWindow {
    let w = window(dark, size)
    let v = NSVisualEffectView()
    v.material = m
    v.blendingMode = .behindWindow
    v.state = .followsWindowActiveState
    w.contentView = v
    return w
}

func plain(_ dark: Bool, _ size: NSSize) -> NSWindow {
    let w = window(dark, size)
    w.backgroundColor = .windowBackgroundColor
    w.contentView = NSView()
    return w
}

func split(_ dark: Bool, _ size: NSSize) -> NSWindow {
    let w = window(dark, size)
    let sc = NSSplitViewController()
    let a = NSViewController(); a.view = NSView()
    let b = NSViewController(); b.view = NSView()
    let side = NSSplitViewItem(sidebarWithViewController: a)
    side.minimumThickness = 200; side.maximumThickness = 200
    sc.addSplitViewItem(side)
    sc.addSplitViewItem(NSSplitViewItem(viewController: b))
    w.contentViewController = sc
    w.setContentSize(size)
    return w
}

func glass(_ dark: Bool, _ size: NSSize) -> NSWindow {
    let w = window(dark, size)
    w.backgroundColor = .clear
    w.isOpaque = false
    let host = NSView()
    let g = NSGlassEffectView()
    g.frame = NSRect(x: 20, y: 20, width: size.width - 40, height: size.height - 40)
    g.autoresizingMask = [.width, .height]
    g.contentView = NSView()
    host.addSubview(g)
    w.contentView = host
    return w
}

func backdrop() -> NSWindow {
    let w = NSWindow(contentRect: screen.frame, styleMask: .borderless, backing: .buffered, defer: false)
    w.level = NSWindow.Level(rawValue: 900)
    w.backgroundColor = NSColor(srgbRed: 0.5, green: 0.5, blue: 0.5, alpha: 1)
    w.hasShadow = false
    w.ignoresMouseEvents = true
    w.isReleasedWhenClosed = false
    w.orderFrontRegardless()
    return w
}

func kinds(_ dark: Bool, _ size: NSSize) -> [(String, NSWindow)] {
    materials.map { ($0.0, vev(dark, $0.1, size)) } + [("plain", plain(dark, size)), ("split", split(dark, size)), ("glass", glass(dark, size))]
}

func activate() { NSApp.activate(); NSApp.activate(ignoringOtherApps: true); pump(0.6) }
func resign() { NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.finder").first?.activate(); pump(0.8) }

func grid() {
    let size = NSSize(width: 520, height: 240)
    for dark in [false, true] {
        let ap = dark ? "dark" : "aqua"
        let ws = kinds(dark, size)
        for (i, (_, w)) in ws.enumerated() {
            place(w, CGFloat(120 + (i % 4) * 590), CGFloat(560 + (i / 4) * 280))
            w.orderFrontRegardless()
        }
        for (label, w) in ws { meta += "\(tag) \(ap) \(label) \(tl(w))\n" }
        activate()
        for (label, w) in ws {
            w.makeKeyAndOrderFront(nil)
            lights(label, w)
            shoot("\(tag)-\(ap)-key-\(label)")
        }
        resign()
        lights("inactive", ws[0].1)
        shoot("\(tag)-\(ap)-inactive")
        for (_, w) in ws { w.orderOut(nil) }
    }
}

func spatial() {
    let size = NSSize(width: 1000, height: 300)
    for dark in [false, true] {
        let ap = dark ? "dark" : "aqua"
        let ws = [("plain", plain(dark, size)), ("sidebar", vev(dark, .sidebar, size))]
        activate()
        for x in stride(from: 0, through: 1560, by: 120) {
            for (i, (label, w)) in ws.enumerated() {
                place(w, CGFloat(x), CGFloat(560 + i * 380))
                w.orderFrontRegardless()
                meta += "\(tag) \(ap) \(label) x\(x) \(tl(w))\n"
            }
            ws[0].1.makeKeyAndOrderFront(nil)
            shoot("\(tag)-\(ap)-x\(x)")
        }
        for (_, w) in ws { w.orderOut(nil) }
    }
}

func hold() {
    let size = NSSize(width: 600, height: 300)
    let dark = tag.contains("dark")
    let w = plain(dark, size)
    let s = vev(dark, .sidebar, size)
    place(w, 400, 600); place(s, 400, 950)
    w.orderFrontRegardless(); s.orderFrontRegardless()
    activate(); w.makeKeyAndOrderFront(nil)
    let go = "\(dir)/move"
    let stop = "\(dir)/stop"
    FileManager.default.createFile(atPath: "\(dir)/ready", contents: nil)
    while !FileManager.default.fileExists(atPath: stop) {
        if FileManager.default.fileExists(atPath: go) {
            let x = Double(((try? String(contentsOfFile: go, encoding: .utf8)) ?? "400").trimmingCharacters(in: .whitespacesAndNewlines)) ?? 400
            try? FileManager.default.removeItem(atPath: go)
            place(w, CGFloat(x), 600); place(s, CGFloat(x), 950)
            CATransaction.flush()
            meta += "move \(x) \(CACurrentMediaTime())\n"
        }
        pump(0.002)
    }
    try? FileManager.default.removeItem(atPath: stop)
}

NSApplication.shared.setActivationPolicy(.regular)
meta += "tintDefault \(UserDefaults.standard.object(forKey: "AppleReduceDesktopTinting") ?? "nil")\n"
let bd = useBackdrop ? backdrop() : nil
switch mode {
case "grid": grid()
case "spatial": spatial()
default: hold()
}
bd?.orderOut(nil)
try? meta.write(toFile: "\(dir)/\(tag).meta", atomically: true, encoding: .utf8)
