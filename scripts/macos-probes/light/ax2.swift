import AppKit
let bid = CommandLine.arguments[1]
let pid = NSRunningApplication.runningApplications(withBundleIdentifier: bid).first!.processIdentifier
let e = AXUIElementCreateApplication(pid)
var names: CFArray?
AXUIElementCopyAttributeNames(e, &names)
print(names as Any)
var v: AnyObject?
print(AXUIElementCopyAttributeValue(e, "AXExtrasMenuBar" as CFString, &v).rawValue, v as Any)
