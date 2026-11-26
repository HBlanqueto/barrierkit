import AppKit

let dir = ProcessInfo.processInfo.environment["PROBE_DIR"]!
let tag = ProcessInfo.processInfo.environment["PROBE_TAG"] ?? "a"
let colorBase = Int(ProcessInfo.processInfo.environment["PROBE_COLOR"] ?? "0")!
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let screen = NSScreen.main!
let H = screen.frame.height
let palette: [[CGFloat]] = [[1, 0, 0], [0, 1, 0], [0, 0, 1], [1, 1, 0], [1, 0, 1], [0, 1, 1]]
var wins: [NSWindow] = []
let logURL = URL(fileURLWithPath: "\(dir)/log-\(tag)")
FileManager.default.createFile(atPath: logURL.path, contents: nil)
let logFH = try! FileHandle(forWritingTo: logURL)

func log(_ s: String) {
  logFH.seekToEndOfFile()
  logFH.write(String(format: "%.6f %@ %@\n", CACurrentMediaTime(), tag, s).data(using: .utf8)!)
}

func rect(_ f: NSRect) -> String { "\(Int(f.minX)) \(Int(H - f.maxY)) \(Int(f.width)) \(Int(f.height))" }

func index(_ w: NSWindow?) -> Int { w.flatMap { x in wins.firstIndex { $0 === x } } ?? -1 }

func window(_ s: [CGFloat]) -> NSWindow {
  let i = wins.count
  let frame = NSRect(x: s[0], y: H - s[1] - s[3], width: s[2], height: s[3])
  let w = NSWindow(contentRect: frame, styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
  w.title = "FS \(tag)\(i)"
  let c = palette[(colorBase + i) % palette.count]
  w.backgroundColor = NSColor(colorSpace: screen.colorSpace!, components: [c[0], c[1], c[2], 1], count: 4)
  w.isReleasedWhenClosed = false
  if s.count >= 6 {
    if s.count > 6 && s[6] == 1 { w.contentMaxSize = NSSize(width: s[4], height: s[5]) }
    w.collectionBehavior.insert(.fullScreenPrimary)
    w.maxFullScreenContentSize = NSSize(width: s[4], height: s[5])
  }
  w.setFrame(frame, display: false)
  wins.append(w)
  return w
}

func state() {
  for (i, w) in wins.enumerated() {
    log("st \(i) wid \(w.windowNumber) frame \(rect(w.frame)) content \(rect(w.contentLayoutRect)) fs \(w.styleMask.contains(.fullScreen)) active \(w.isOnActiveSpace) visible \(w.isVisible) key \(w.isKeyWindow) main \(w.isMainWindow)")
  }
}

func dump(_ m: NSMenu, _ d: String) {
  for it in m.items {
    log("menu \(d)\(it.title) | \(it.action.map { NSStringFromSelector($0) } ?? "-") enabled \(it.isEnabled)")
    if let s = it.submenu { s.update(); dump(s, d + "  ") }
  }
}

func press(_ path: [String]) {
  var m: NSMenu? = app.mainMenu
  for (k, name) in path.enumerated() {
    m?.update()
    guard let it = m?.items.first(where: { $0.title.contains(name) }) else { log("menu-miss \(name)"); return }
    if k == path.count - 1 {
      let idx = m!.index(of: it)
      log("menu-press \(it.title)")
      m!.performActionForItem(at: idx)
    } else { m = it.submenu }
  }
}

func run(_ line: String) {
  let p = line.split(separator: " ", maxSplits: 1).map(String.init)
  let arg = p.count > 1 ? p[1] : ""
  let n = Int(arg) ?? 0
  log("cmd \(line)")
  switch p[0] {
  case "fs": wins[n].toggleFullScreen(nil)
  case "new", "newx":
    let w = window(arg.split(separator: ",").map { CGFloat(Double($0)!) })
    if p[0] == "newx" { w.tabbingMode = .disallowed }
    w.makeKeyAndOrderFront(nil); log("new \(wins.count - 1) wid \(w.windowNumber)")
  case "close": wins[n].close()
  case "key": wins[n].makeKeyAndOrderFront(nil)
  case "act": app.activate(ignoringOtherApps: true)
  case "st": state()
  case "menus": dump(app.mainMenu!, "")
  case "menu": press(arg.split(separator: ">").map(String.init))
  case "quit": exit(0)
  default: log("bad \(line)")
  }
}

final class Obs: NSObject {
  @objc func note(_ n: Notification) {
    let w = n.object as? NSWindow
    var s = "\(n.name.rawValue) \(index(w))"
    if let w = w { s += " frame \(rect(w.frame))" }
    log(s)
  }
}
let obs = Obs()
let names: [Notification.Name] = [NSWindow.willEnterFullScreenNotification, NSWindow.didEnterFullScreenNotification, NSWindow.willExitFullScreenNotification, NSWindow.didExitFullScreenNotification, NSWindow.didBecomeKeyNotification, NSWindow.didResignKeyNotification, NSWindow.didBecomeMainNotification, NSWindow.didResignMainNotification, NSWindow.didResizeNotification, NSWindow.didMoveNotification, NSWindow.didChangeOcclusionStateNotification, NSWindow.willCloseNotification, NSApplication.didBecomeActiveNotification, NSApplication.didResignActiveNotification]
for nm in names { NotificationCenter.default.addObserver(obs, selector: #selector(Obs.note(_:)), name: nm, object: nil) }
NSWorkspace.shared.notificationCenter.addObserver(obs, selector: #selector(Obs.note(_:)), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)

let mainMenu = NSMenu()
let appItem = NSMenuItem(); appItem.submenu = NSMenu(); mainMenu.addItem(appItem)
let winItem = NSMenuItem(title: "Window", action: nil, keyEquivalent: "")
let winMenu = NSMenu(title: "Window")
winMenu.addItem(withTitle: "Minimize", action: #selector(NSWindow.performMiniaturize(_:)), keyEquivalent: "m")
winItem.submenu = winMenu
mainMenu.addItem(winItem)
app.mainMenu = mainMenu
app.windowsMenu = winMenu

let cmdPath = "\(dir)/cmd-\(tag)"
var lastSeq = -1
DispatchQueue.main.async {
  app.activate(ignoringOtherApps: true)
  for spec in CommandLine.arguments.dropFirst() { window(spec.split(separator: ",").map { CGFloat(Double($0)!) }).makeKeyAndOrderFront(nil) }
  state()
  FileManager.default.createFile(atPath: "\(dir)/ready-\(tag)", contents: nil)
  Timer.scheduledTimer(withTimeInterval: 0.002, repeats: true) { _ in
    for t in ["fs0", "fs1", "close0", "close1"] {
      let f = "\(dir)/t-\(tag)-\(t)"
      if FileManager.default.fileExists(atPath: f) { try? FileManager.default.removeItem(atPath: f); run(t.hasPrefix("fs") ? "fs \(t.dropFirst(2))" : "close \(t.dropFirst(5))") }
    }
    guard let c = try? String(contentsOfFile: cmdPath, encoding: .utf8) else { return }
    let p = c.trimmingCharacters(in: .whitespacesAndNewlines).split(separator: " ", maxSplits: 1).map(String.init)
    guard p.count == 2, let seq = Int(p[0]), seq != lastSeq else { return }
    lastSeq = seq
    for l in p[1].split(separator: ";") { run(l.trimmingCharacters(in: .whitespaces)) }
  }
}
app.run()
