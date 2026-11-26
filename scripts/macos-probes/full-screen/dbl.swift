import AppKit
let x = Double(CommandLine.arguments[1])!, y = Double(CommandLine.arguments[2])!
let src = CGEventSource(stateID: .hidSystemState)
for n in [1, 2] { for t in [CGEventType.leftMouseDown, .leftMouseUp] {
  let e = CGEvent(mouseEventSource: src, mouseType: t, mouseCursorPosition: CGPoint(x: x, y: y), mouseButton: .left)!
  e.setIntegerValueField(.mouseEventClickState, value: Int64(n)); e.post(tap: .cghidEventTap); usleep(40000) } }
