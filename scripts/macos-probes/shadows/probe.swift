import AppKit
import ScreenCaptureKit
import CoreMedia

let out = "/private/tmp/claude-501/-Users-humbe--claude-spares-spare-wxcbfZ/815469d1-10d1-46d0-9155-bfb2393396d4/scratchpad/shadows/data"
try? FileManager.default.createDirectory(atPath: out, withIntermediateDirectories: true)
let screen = NSScreen.main!
let H = screen.frame.height
let topLevel = NSWindow.Level(rawValue: 901)
var meta = ""
var log = ""

func pump(_ s: Double) { RunLoop.current.run(until: Date().addingTimeInterval(s)) }

func color(_ v: CGFloat) -> NSColor { NSColor(colorSpace: screen.colorSpace!, components: [v, v, v, 1], count: 4) }

func backdrop(_ v: CGFloat) -> NSWindow {
    let w = NSWindow(contentRect: screen.frame, styleMask: .borderless, backing: .buffered, defer: false)
    w.level = NSWindow.Level(rawValue: 900)
    w.backgroundColor = color(v)
    w.hasShadow = false
    w.ignoresMouseEvents = true
    w.isReleasedWhenClosed = false
    w.orderFrontRegardless()
    return w
}

func rectTL(_ w: NSWindow) -> String {
    let f = w.frame
    return "\(Int(f.minX)) \(Int(H - f.maxY)) \(Int(f.width)) \(Int(f.height))"
}

func shoot(_ name: String, _ wins: [(String, NSWindow)]) {
    pump(0.25)
    for (label, w) in wins {
        meta += "\(name) \(label) \(rectTL(w)) key=\(w.isKeyWindow) active=\(NSApp.isActive)\n"
    }
    let png = "\(out)/\(name).png"
    FileManager.default.createFile(atPath: "\(out)/request", contents: Data(png.utf8))
    while FileManager.default.fileExists(atPath: "\(out)/request") { pump(0.01) }
}

let mouse = NSEvent.mouseLocation
let mouseTL = CGPoint(x: mouse.x, y: H - mouse.y)
let slotOrigins: [CGPoint] = [CGPoint(x: 100, y: 150), CGPoint(x: 1860, y: 150), CGPoint(x: 100, y: 890), CGPoint(x: 1860, y: 890)]
let freeSlots = slotOrigins.filter { !CGRect(x: $0.x - 120, y: $0.y - 120, width: 840, height: 640).contains(mouseTL) }

func place(_ w: NSWindow, _ slot: CGPoint) {
    w.setFrame(NSRect(x: slot.x, y: H - slot.y - w.frame.height, width: w.frame.width, height: w.frame.height), display: true)
}

func titled(_ dark: Bool, _ size: NSSize = NSSize(width: 600, height: 400)) -> NSWindow {
    let w = NSWindow(contentRect: NSRect(origin: .zero, size: size), styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
    w.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
    w.title = "Probe"
    w.level = topLevel
    w.isReleasedWhenClosed = false
    w.setFrame(NSRect(origin: .zero, size: size), display: false)
    return w
}

func borderless(_ dark: Bool) -> NSWindow {
    let w = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: .borderless, backing: .buffered, defer: false)
    w.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
    w.backgroundColor = .windowBackgroundColor
    w.isOpaque = true
    w.hasShadow = true
    w.level = topLevel
    w.isReleasedWhenClosed = false
    return w
}

func panel(_ dark: Bool) -> NSPanel {
    let p = NSPanel(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled, .closable, .utilityWindow, .nonactivatingPanel], backing: .buffered, defer: false)
    p.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
    p.title = "Panel"
    p.level = topLevel
    p.becomesKeyOnlyIfNeeded = false
    p.isReleasedWhenClosed = false
    p.setFrame(NSRect(x: 0, y: 0, width: 600, height: 400), display: false)
    return p
}

func activate() {
    NSApp.activate(ignoringOtherApps: true)
    pump(0.3)
}

func shadows(_ borderlessOnly: Bool) {
    activate()
    log += "mouse TL \(mouseTL) freeSlots \(freeSlots.count)\n"
    let kinds: [(String, NSWindow)] = borderlessOnly ? [("borderless-aqua", borderless(false)), ("borderless-dark", borderless(true))]
        : [("titled-aqua", titled(false)), ("titled-dark", titled(true)), ("panel-aqua", panel(false)), ("panel-dark", panel(true))]
    let wins = Array(zip(kinds, freeSlots).map { $0.0 })
    for ((_, w), slot) in zip(wins, freeSlots) { place(w, slot) }
    for (name, v) in [("white", 1.0), ("gray", 128.0 / 255.0), ("black", 0.0)] {
        let b = backdrop(CGFloat(v))
        for (_, w) in wins { w.orderFrontRegardless() }
        wins[0].1.makeKeyAndOrderFront(nil)
        shoot("\(borderlessOnly ? "bl" : "ti")-\(name)-A", wins)
        if !borderlessOnly { wins[1].1.makeKeyAndOrderFront(nil); shoot("\(borderlessOnly ? "bl" : "ti")-\(name)-B", wins) }
        if let p = wins.first(where: { $0.0 == "panel-aqua" }) { p.1.makeKeyAndOrderFront(nil); shoot("\(borderlessOnly ? "bl" : "ti")-\(name)-C", wins) }
        for (_, w) in wins { w.orderOut(nil) }
        b.orderOut(nil)
        pump(1.0)
    }
}

func modals() {
    activate()
    for (name, v) in [("white", 1.0), ("gray", 128.0 / 255.0)] {
        for dark in [false, true] {
            let tag = "\(name)-\(dark ? "dark" : "aqua")"
            let b = backdrop(CGFloat(v))
            let p1 = titled(dark, NSSize(width: 800, height: 700))
            let p2 = titled(dark, NSSize(width: 800, height: 700))
            for p in [p1, p2] { p.backgroundColor = color(CGFloat(v)) }
            p1.setFrameOrigin(NSPoint(x: 100, y: H - 120 - 700))
            p2.setFrameOrigin(NSPoint(x: 1700, y: H - 120 - 700))
            p1.orderFrontRegardless(); p2.makeKeyAndOrderFront(nil)
            let sheet = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled], backing: .buffered, defer: false)
            sheet.appearance = p1.appearance
            sheet.isReleasedWhenClosed = false
            p1.beginSheet(sheet)
            let alert = NSAlert()
            alert.messageText = "Probe"
            alert.window.appearance = p2.appearance
            alert.beginSheetModal(for: p2)
            pump(0.6)
            shoot("\(tag)-S", [("sheet-parent", p1), ("sheet", sheet), ("alertsheet-parent", p2), ("alertsheet", alert.window)])
            p1.endSheet(sheet); p2.endSheet(alert.window)
            p1.orderOut(nil); p2.orderOut(nil)
            let anchor = titled(dark, NSSize(width: 300, height: 120))
            anchor.setFrameOrigin(NSPoint(x: 300, y: 200))
            anchor.makeKeyAndOrderFront(nil)
            let pop = NSPopover()
            let vc = NSViewController()
            vc.view = NSView(frame: NSRect(x: 0, y: 0, width: 400, height: 300))
            pop.contentViewController = vc
            pop.appearance = anchor.appearance
            pop.show(relativeTo: NSRect(x: 140, y: 40, width: 20, height: 20), of: anchor.contentView!, preferredEdge: .maxY)
            pump(0.3)
            let popWin = vc.view.window!
            popWin.level = NSWindow.Level(rawValue: 902)
            let modalAlert = NSAlert()
            modalAlert.messageText = "Probe"
            modalAlert.window.appearance = anchor.appearance
            let mw = modalAlert.window
            DispatchQueue.global().async {
                Thread.sleep(forTimeInterval: 0.5)
                DispatchQueue.main.sync { mw.level = NSWindow.Level(rawValue: 903); mw.setFrameOrigin(NSPoint(x: 1800, y: 250)) }
                DispatchQueue.main.sync { shoot("\(tag)-M", [("popover", popWin), ("anchor", anchor), ("alert", mw)]) }
                DispatchQueue.main.async { NSApp.abortModal() }
            }
            modalAlert.runModal()
            mw.orderOut(nil)
            pop.close()
            anchor.orderOut(nil)
            b.orderOut(nil)
            pump(1.0)
        }
    }
}

final class Tracker: NSObject, SCStreamOutput {
    var rows = ""
    var reference: [UInt8] = []
    var takeReference = false
    let originY: Int
    init(originY: Int) { self.originY = originY }
    func stream(_ stream: SCStream, didOutputSampleBuffer sb: CMSampleBuffer, of type: SCStreamOutputType) {
        guard type == .screen, let pb = CMSampleBufferGetImageBuffer(sb) else { return }
        CVPixelBufferLockBaseAddress(pb, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(pb, .readOnly) }
        let w = CVPixelBufferGetWidth(pb), h = CVPixelBufferGetHeight(pb), bpr = CVPixelBufferGetBytesPerRow(pb)
        let base = CVPixelBufferGetBaseAddress(pb)!.assumingMemoryBound(to: UInt8.self)
        if takeReference { reference = Array(UnsafeBufferPointer(start: base, count: bpr * h)); takeReference = false }
        guard !reference.isEmpty else { return }
        var top = -1, bottom = -1, left = Int.max, right = -1, count = 0
        for y in 0..<h {
            let row = y * bpr
            for x in 0..<w {
                let i = row + x * 4
                if abs(Int(base[i]) - Int(reference[i])) + abs(Int(base[i + 1]) - Int(reference[i + 1])) + abs(Int(base[i + 2]) - Int(reference[i + 2])) > 60 {
                    if top < 0 { top = y }
                    bottom = y; left = min(left, x); right = max(right, x); count += 1
                }
            }
        }
        let t = CMSampleBufferGetPresentationTimeStamp(sb).seconds
        rows += String(format: "%.5f %d %d %d %d %d\n", t, top < 0 ? -1 : top + originY, bottom < 0 ? -1 : bottom + originY, left, right, count)
    }
}

func now() -> Double { CMClockGetTime(CMClockGetHostTimeClock()).seconds }

func dock() {
    let img = NSImage(size: NSSize(width: 512, height: 512), flipped: false) { r in
        NSColor(srgbRed: 1, green: 0, blue: 1, alpha: 1).setFill(); r.fill(); return true
    }
    NSApp.applicationIconImage = img
    pump(2.0)
    let sem = DispatchSemaphore(value: 0)
    var content: SCShareableContent?
    SCShareableContent.getWithCompletionHandler { c, e in content = c; if let e { log += "sck error \(e)\n" }; sem.signal() }
    while sem.wait(timeout: .now()) == .timedOut { pump(0.01) }
    guard let display = content?.displays.first else { log += "no display\n"; return }
    let regionH = 300
    let filter = SCContentFilter(display: display, excludingWindows: [])
    let cfg = SCStreamConfiguration()
    cfg.sourceRect = CGRect(x: 0, y: Int(H) - regionH, width: Int(screen.frame.width), height: regionH)
    cfg.width = Int(screen.frame.width); cfg.height = regionH
    cfg.minimumFrameInterval = CMTime(value: 1, timescale: 240)
    cfg.pixelFormat = kCVPixelFormatType_32BGRA
    cfg.showsCursor = false
    cfg.queueDepth = 8
    let tracker = Tracker(originY: Int(H) - regionH)
    let stream = SCStream(filter: filter, configuration: cfg, delegate: nil)
    try! stream.addStreamOutput(tracker, type: .screen, sampleHandlerQueue: DispatchQueue(label: "sck"))
    var started = false
    stream.startCapture { e in if let e { log += "start error \(e)\n" }; started = true }
    while !started { pump(0.01) }
    pump(1.0)
    tracker.takeReference = true
    pump(0.5)
    log += String(format: "active=%d\ncritical %.5f\n", NSApp.isActive ? 1 : 0, now())
    let req = NSApp.requestUserAttention(.criticalRequest)
    pump(6.0)
    NSApp.cancelUserAttentionRequest(req)
    log += String(format: "cancel %.5f\n", now())
    pump(2.0)
    log += String(format: "informational %.5f\n", now())
    _ = NSApp.requestUserAttention(.informationalRequest)
    pump(4.0)
    var stopped = false
    stream.stopCapture { _ in stopped = true }
    while !stopped { pump(0.01) }
    try! tracker.rows.write(toFile: "\(out)/dock.txt", atomically: true, encoding: .utf8)
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)
switch CommandLine.arguments.dropFirst().first ?? "" {
case "shadows", "borderless": shadows(CommandLine.arguments[1] == "borderless"); try! meta.write(toFile: "\(out)/meta-\(CommandLine.arguments[1]).txt", atomically: true, encoding: .utf8)
case "modals": modals(); try! meta.write(toFile: "\(out)/meta-modals.txt", atomically: true, encoding: .utf8)
case "dock": dock()
default: print("usage: probe shadows|modals|dock")
}
print(log, terminator: "")
exit(0)
