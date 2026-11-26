import AppKit
import ApplicationServices

// ui extras <bundle-id>            -> list the extras menu bar items: index, description, frame
// ui press <bundle-id> <index> [extras|menubar]
// ui key <keycode> [cmd]
// ui click x y
// ui wins                          -> on-screen windows: owner layer x y w h
let a = CommandLine.arguments
func attr(_ e: AXUIElement, _ k: String) -> AnyObject? { var v: AnyObject?; AXUIElementCopyAttributeValue(e, k as CFString, &v); return v }
func app(_ bid: String) -> AXUIElement {
  let pid = NSRunningApplication.runningApplications(withBundleIdentifier: bid).first!.processIdentifier
  let e = AXUIElementCreateApplication(pid); AXUIElementSetMessagingTimeout(e, 1.0); return e
}
func items(_ e: AXUIElement, _ bar: String) -> [AXUIElement] {
  guard let b = attr(e, bar) else { return [] }
  return (attr(b as! AXUIElement, "AXChildren") as? [AXUIElement]) ?? []
}
func frame(_ e: AXUIElement) -> CGRect {
  var p = CGPoint.zero, s = CGSize.zero
  if let v = attr(e, "AXPosition") { AXValueGetValue(v as! AXValue, .cgPoint, &p) }
  if let v = attr(e, "AXSize") { AXValueGetValue(v as! AXValue, .cgSize, &s) }
  return CGRect(origin: p, size: s)
}
func post(_ t: CGEventType, _ x: Double, _ y: Double) {
  CGEvent(mouseEventSource: CGEventSource(stateID: .hidSystemState), mouseType: t, mouseCursorPosition: CGPoint(x: x, y: y), mouseButton: .left)!.post(tap: .cghidEventTap)
}
switch a[1] {
case "extras", "menubar":
  for (i, it) in items(app(a[2]), a[1] == "extras" ? "AXExtrasMenuBar" : "AXMenuBar").enumerated() {
    print(i, attr(it, "AXDescription") as? String ?? "", attr(it, "AXTitle") as? String ?? "", attr(it, "AXIdentifier") as? String ?? "", frame(it))
  }
case "press":
  var it = items(app(a[2]), a.count > 4 && a[4] == "menubar" ? "AXMenuBar" : "AXExtrasMenuBar")[Int(a[3])!]
  if attr(it, "AXRole") as? String == "AXGroup" { it = (attr(it, "AXChildren") as! [AXUIElement])[0] }
  print(AXUIElementPerformAction(it, (a.count > 5 ? a[5] : "AXPress") as CFString).rawValue)
case "key":
  let src = CGEventSource(stateID: .hidSystemState)
  for down in [true, false] {
    let e = CGEvent(keyboardEventSource: src, virtualKey: CGKeyCode(Int(a[2])!), keyDown: down)!
    if a.count > 3 { e.flags = .maskCommand }
    e.post(tap: .cghidEventTap); usleep(30000)
  }
case "click":
  let x = Double(a[2])!, y = Double(a[3])!
  post(.mouseMoved, x, y); usleep(50000); post(.leftMouseDown, x, y); usleep(50000); post(.leftMouseUp, x, y)
case "move": post(.mouseMoved, Double(a[2])!, Double(a[3])!)
case "wins":
  for w in CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]] {
    let b = w[kCGWindowBounds as String] as! [String: Double]
    print(w[kCGWindowOwnerName as String] as? String ?? "?", w[kCGWindowLayer as String] as! Int, Int(b["X"]!), Int(b["Y"]!), Int(b["Width"]!), Int(b["Height"]!), w[kCGWindowName as String] as? String ?? "")
  }
default: break
}
