import AppKit

let dir = ProcessInfo.processInfo.environment["PROBE_DIR"]!
let specs = CommandLine.arguments.dropFirst().map { $0.split(separator: ",").map { CGFloat(Double($0)!) } }
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let screen = NSScreen.main!
let H = screen.frame.height
let palette: [[CGFloat]] = [[1, 0, 0], [0, 1, 0], [0, 0, 1], [1, 1, 0], [0, 1, 1], [1, 0, 1], [1, 0.5, 0], [0.5, 0, 1], [0, 0.5, 0.25], [0.5, 0.5, 0.5], [1, 1, 1]]
var wins: [NSWindow] = []
let env = ProcessInfo.processInfo.environment
let first = Int(env["PROBE_FIRST"] ?? "0")!
let raised = (env["PROBE_RAISE"] ?? "").split(separator: ",").map { Int($0)! }

func window(_ s: [CGFloat], _ i: Int) -> NSWindow {
  let style: NSWindow.StyleMask = s.count > 4 && s[4] == 0 ? .borderless : [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView]
  let frame = NSRect(x: s[0], y: H - s[1] - s[3], width: s[2], height: s[3])
  let w = NSWindow(contentRect: frame, styleMask: style, backing: .buffered, defer: false)
  w.titlebarAppearsTransparent = true
  w.titleVisibility = .hidden
  w.title = "Probe \(i + 1)"
  let c = palette[(i + first) % palette.count]
  w.backgroundColor = NSColor(colorSpace: screen.colorSpace!, components: [c[0], c[1], c[2], 1], count: 4)
  w.isReleasedWhenClosed = false
  w.setFrame(frame, display: false)
  return w
}

DispatchQueue.main.async {
  app.activate(ignoringOtherApps: true)
  for (i, s) in specs.enumerated() { let w = window(s, i); w.makeKeyAndOrderFront(nil); wins.append(w) }
  for i in raised { wins[i].makeKeyAndOrderFront(nil) }
  FileManager.default.createFile(atPath: "\(dir)/ready", contents: nil)
  Timer.scheduledTimer(withTimeInterval: 0.05, repeats: true) { _ in
    if FileManager.default.fileExists(atPath: "\(dir)/quit") { try? FileManager.default.removeItem(atPath: "\(dir)/quit"); exit(0) }
  }
}
app.run()
