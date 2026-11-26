import AppKit

let a = CommandLine.arguments
let app = AXUIElementCreateApplication(pid_t(a[1])!)
let path = Array(a.dropFirst(2))
func g(_ x: AXUIElement, _ k: String) -> AnyObject? { var v: AnyObject?; AXUIElementCopyAttributeValue(x, k as CFString, &v); return v }
func kids(_ x: AXUIElement) -> [AXUIElement] { (g(x, "AXChildren") as? [AXUIElement]) ?? [] }
func title(_ x: AXUIElement) -> String { (g(x, "AXTitle") as? String) ?? "" }
func items(_ x: AXUIElement) -> [AXUIElement] { kids(x).flatMap { title($0).isEmpty && (g($0, "AXRole") as? String) == "AXMenu" ? kids($0) : [$0] } }
func dump(_ x: AXUIElement, _ d: String) {
  for it in items(x) {
    print("\(d)\(title(it)) enabled \((g(it, "AXEnabled") as? Bool) ?? false)")
    if d.count < 6 { for m in kids(it) where (g(m, "AXRole") as? String) == "AXMenu" { dump(m, d + "  ") } }
  }
}
print("trusted", AXIsProcessTrusted())
var cur = g(app, "AXMenuBar") as! AXUIElement
for (i, name) in path.enumerated() {
  guard let it = items(cur).first(where: { title($0) == name }) else { print("miss", name); dump(cur, "  "); exit(1) }
  if i == path.count - 1 && i > 0 { print(String(format: "%.6f press %@", CACurrentMediaTime(), name)); AXUIElementPerformAction(it, "AXPress" as CFString); exit(0) }
  if i == 0 { AXUIElementPerformAction(it, "AXPress" as CFString); usleep(300000) }
  guard let m = kids(it).first(where: { (g($0, "AXRole") as? String) == "AXMenu" }) else { print("no submenu", name); exit(1) }
  cur = m
}
dump(cur, "  ")
