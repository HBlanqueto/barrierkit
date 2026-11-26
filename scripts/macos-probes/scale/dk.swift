import AppKit
let s = NSScreen.main!
print("screen", Int(s.frame.width), Int(s.frame.height), "scale", s.backingScaleFactor, "visible", Int(s.visibleFrame.minY), Int(s.visibleFrame.height), "refresh", CGDisplayCopyDisplayMode(CGMainDisplayID())!.refreshRate, "pxw", CGDisplayCopyDisplayMode(CGMainDisplayID())!.pixelWidth)
let pid = NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.dock").first!.processIdentifier
let e = AXUIElementCreateApplication(pid)
func g(_ x: AXUIElement, _ k: String) -> AnyObject? { var v: AnyObject?; AXUIElementCopyAttributeValue(x, k as CFString, &v); return v }
for c in (g(e, "AXChildren") as? [AXUIElement]) ?? [] where g(c, "AXRole") as? String == "AXList" {
  var p = CGPoint.zero, z = CGSize.zero
  AXValueGetValue(g(c, "AXPosition") as! AXValue, .cgPoint, &p); AXValueGetValue(g(c, "AXSize") as! AXValue, .cgSize, &z)
  print("docklist", p.x, p.y, z.width, z.height)
}
