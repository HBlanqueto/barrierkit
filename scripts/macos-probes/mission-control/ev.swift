import AppKit

func key(_ code: UInt16, _ ctrl: Bool) {
  let src = CGEventSource(stateID: .hidSystemState)
  for down in [true, false] {
    let e = CGEvent(keyboardEventSource: src, virtualKey: code, keyDown: down)!
    if ctrl { e.flags = [.maskControl, .maskSecondaryFn, .maskNumericPad] }
    e.post(tap: .cghidEventTap)
  }
}
func mouse(_ t: CGEventType, _ x: Double, _ y: Double) {
  let e = CGEvent(mouseEventSource: CGEventSource(stateID: .hidSystemState), mouseType: t, mouseCursorPosition: CGPoint(x: x, y: y), mouseButton: .left)!
  e.setIntegerValueField(.mouseEventClickState, value: 1); e.post(tap: .cghidEventTap)
}
for act in CommandLine.arguments.dropFirst() {
  let p = act.split(separator: ":").map(String.init)
  let n = p.count > 1 ? p[1].split(separator: ",").compactMap { Double($0) } : []
  switch p[0] {
  case "k": key(UInt16(n[0]), true)
  case "kp": key(UInt16(n[0]), false)
  case "m": mouse(.mouseMoved, n[0], n[1])
  case "c": mouse(.leftMouseDown, n[0], n[1]); usleep(60000); mouse(.leftMouseUp, n[0], n[1])
  case "s": usleep(UInt32(n[0] * 1000))
  case "where": let l = NSEvent.mouseLocation; print(l.x, NSScreen.main!.frame.height - l.y)
  default: print("bad", act)
  }
}
