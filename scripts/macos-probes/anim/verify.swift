import AppKit
let l = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
print("Claude layer0 on screen:", l.contains { ($0[kCGWindowOwnerName as String] as? String) == "Claude" && ($0[kCGWindowLayer as String] as? Int) == 0 })
print("probe windows:", l.filter { ["AnimProbe","probe","red","green","blue","yellow","cyan"].contains(($0[kCGWindowOwnerName as String] as? String) ?? "") }.count)
print("visibleFrame", NSScreen.main!.visibleFrame)
