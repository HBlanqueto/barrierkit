import AppKit
let pid = NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.MenuBarAgent").first!.processIdentifier
let e = AXUIElementCreateApplication(pid)
func g(_ x: AXUIElement, _ k: String) -> AnyObject? { var v: AnyObject?; AXUIElementCopyAttributeValue(x, k as CFString, &v); return v }
func walk(_ x: AXUIElement, _ d: Int, _ path: String) {
  var n: CFArray?; AXUIElementCopyActionNames(x, &n)
  print(String(repeating: " ", count: d), path, g(x, "AXRole") ?? "", g(x, "AXDescription") ?? "", g(x, "AXIdentifier") ?? "", (n as? [String]) ?? [])
  for (i, c) in ((g(x, "AXChildren") as? [AXUIElement]) ?? []).enumerated() where d < 4 { walk(c, d + 1, path + ".\(i)") }
}
walk(g(e, "AXExtrasMenuBar") as! AXUIElement, 0, "")
