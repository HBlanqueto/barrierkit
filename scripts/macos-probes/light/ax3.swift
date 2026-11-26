import AppKit
for a in NSWorkspace.shared.runningApplications {
  let e = AXUIElementCreateApplication(a.processIdentifier)
  AXUIElementSetMessagingTimeout(e, 0.3)
  var v: AnyObject?
  if AXUIElementCopyAttributeValue(e, "AXExtrasMenuBar" as CFString, &v) == .success, let bar = v {
    var c: AnyObject?
    AXUIElementCopyAttributeValue(bar as! AXUIElement, "AXChildren" as CFString, &c)
    for it in (c as? [AXUIElement]) ?? [] {
      var d: AnyObject?, p: AnyObject?
      AXUIElementCopyAttributeValue(it, "AXDescription" as CFString, &d)
      AXUIElementCopyAttributeValue(it, "AXPosition" as CFString, &p)
      print(a.bundleIdentifier ?? "?", d as? String ?? "", p.map { "\($0)" }?.replacingOccurrences(of: "\n", with: " ") ?? "")
    }
  }
}
