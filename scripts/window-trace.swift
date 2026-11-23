// Prints every change of the bounds of a process's main window for a few seconds, so a script can tell
// a window that animates from one that jumps. Usage: window-trace <pid> <seconds>
// Output: one "<ms> <x> <y> <w> <h>" line per distinct bounds.
import CoreGraphics
import Foundation

let args = CommandLine.arguments
guard args.count == 3, let pid = Int32(args[1]), let seconds = Double(args[2]) else {
    FileHandle.standardError.write(Data("usage: window-trace pid seconds\n".utf8))
    exit(2)
}
let start = Date()
var last = ""
while Date().timeIntervalSince(start) < seconds {
    let all = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] ?? []
    let mine = all.first { ($0[kCGWindowOwnerPID as String] as? Int32) == pid && ($0[kCGWindowLayer as String] as? Int) == 0 }
    if let b = mine?[kCGWindowBounds as String] as? [String: Double] {
        let line = "\(Int(b["X"] ?? 0)) \(Int(b["Y"] ?? 0)) \(Int(b["Width"] ?? 0)) \(Int(b["Height"] ?? 0))"
        if line != last {
            print("\(Int(Date().timeIntervalSince(start) * 1000)) \(line)")
            last = line
        }
    }
    usleep(4000)
}
