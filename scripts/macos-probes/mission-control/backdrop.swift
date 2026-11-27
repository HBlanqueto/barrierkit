import AppKit

let dir = CommandLine.arguments[1]
let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let screen = NSScreen.main!
let window = NSWindow(contentRect: screen.frame, styleMask: .borderless, backing: .buffered, defer: false)
window.level = .normal
window.collectionBehavior = [.canJoinAllSpaces, .stationary, .ignoresCycle]
window.ignoresMouseEvents = true
window.isReleasedWhenClosed = false
window.backgroundColor = .black
window.orderFront(nil)
Timer.scheduledTimer(withTimeInterval: 0.01, repeats: true) { _ in
  let ack = dir + "/ack"
  guard !FileManager.default.fileExists(atPath: ack),
        let raw = try? String(contentsOfFile: dir + "/mode", encoding: .utf8) else { return }
  let mode = raw.trimmingCharacters(in: .whitespacesAndNewlines)
  if mode == "off" { window.orderOut(nil) }
  else {
    let c = mode.dropFirst(4).split(separator: ",").map { CGFloat(Double($0)!) }
    window.backgroundColor = NSColor(colorSpace: screen.colorSpace!, components: [c[0], c[1], c[2], 1], count: 4)
    window.orderFront(nil)
  }
  FileManager.default.createFile(atPath: ack, contents: nil)
}
app.run()
