import Foundation

let sky = dlopen("/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight", RTLD_LAZY)
typealias Conn = @convention(c) () -> Int32
typealias Spaces = @convention(c) (Int32) -> Unmanaged<CFArray>
typealias ForWins = @convention(c) (Int32, Int32, CFArray) -> Unmanaged<CFArray>
let cid = unsafeBitCast(dlsym(sky, "SLSMainConnectionID"), to: Conn.self)()
let all = unsafeBitCast(dlsym(sky, "SLSCopyManagedDisplaySpaces"), to: Spaces.self)(cid).takeRetainedValue() as! [[String: Any]]
for d in all {
  let cur = (d["Current Space"] as? [String: Any])?["ManagedSpaceID"] as? Int ?? -1
  var line = "current \(cur) |"
  for s in d["Spaces"] as? [[String: Any]] ?? [] {
    let id = s["ManagedSpaceID"] as? Int ?? -1
    let type = s["type"] as? Int ?? -1
    var extra = ""
    if let tl = s["TileLayoutManager"] as? [String: Any], let tiles = tl["TileSpaces"] as? [[String: Any]] {
      extra = " tiles " + tiles.map { "\($0["TileWindowID"] ?? $0["ManagedSpaceID"] ?? "?")" }.joined(separator: "+")
    }
    if let w = s["fs_wid"] { extra += " fs_wid \(w)" }
    line += " \(id):t\(type)\(extra)\(id == cur ? "*" : "")"
  }
  print(line)
}
let wids = CommandLine.arguments.dropFirst().compactMap { Int($0) }
if !wids.isEmpty {
  let f = unsafeBitCast(dlsym(sky, "SLSCopySpacesForWindows"), to: ForWins.self)
  for w in wids { print("wid \(w) spaces \(f(cid, 7, [w] as CFArray).takeRetainedValue() as! [Int])") }
}
