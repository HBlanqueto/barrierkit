import AppKit
import ScreenCaptureKit
import CoreMedia
import CoreImage

// usage: cap OUT x y w h outW outH secs action...
// output OUT.bin: per frame Double t, then for classes R G B Y M C DIFF: rows x (Int16 minx, maxx, count), then rows x UInt8 meanLuma
// classes are channel patterns H>170 L<80; DIFF is |px - first frame| sum > 90
let a = CommandLine.arguments
let outPath = a[1]
let src = CGRect(x: Double(a[2])!, y: Double(a[3])!, width: Double(a[4])!, height: Double(a[5])!)
let OW = Int(a[6])!, OH = Int(a[7])!
let secs = Double(a[8])!
let actions = Array(a.dropFirst(9))
let patterns: [(Bool, Bool, Bool)] = [(true, false, false), (false, true, false), (false, false, true), (true, true, false), (true, false, true), (false, true, true)]
let NC = patterns.count + 1
FileManager.default.createFile(atPath: outPath + ".bin", contents: nil)
let fh = FileHandle(forWritingAtPath: outPath + ".bin")!
var evlog = ""
func hostNow() -> Double { CMTimeGetSeconds(CMClockGetTime(CMClockGetHostTimeClock())) }

final class Out: NSObject, SCStreamOutput {
  var first: [UInt8]? = nil
  var n = 0
  var buf = Data()
  func stream(_ s: SCStream, didOutputSampleBuffer sb: CMSampleBuffer, of t: SCStreamOutputType) {
    guard t == .screen, let atts = CMSampleBufferGetSampleAttachmentsArray(sb, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
          let st = atts.first?[.status] as? Int, st == SCFrameStatus.complete.rawValue,
          let pb = CMSampleBufferGetImageBuffer(sb) else { return }
    CVPixelBufferLockBaseAddress(pb, .readOnly); defer { CVPixelBufferUnlockBaseAddress(pb, .readOnly) }
    let w = min(CVPixelBufferGetWidth(pb), OW), h = min(CVPixelBufferGetHeight(pb), OH), bpr = CVPixelBufferGetBytesPerRow(pb)
    let p = CVPixelBufferGetBaseAddress(pb)!.assumingMemoryBound(to: UInt8.self)
    if first == nil { first = [UInt8](UnsafeBufferPointer(start: p, count: bpr * h)) }
    var mn = [Int16](repeating: Int16.max, count: NC * OH), mx = [Int16](repeating: -1, count: NC * OH), ct = [Int16](repeating: 0, count: NC * OH)
    var lum = [UInt8](repeating: 0, count: OH)
    first!.withUnsafeBufferPointer { f in
      for y in 0..<h {
        var ls = 0
        for x in 0..<w {
          let o = y * bpr + x * 4
          let b = Int(p[o]), g = Int(p[o + 1]), r = Int(p[o + 2])
          ls += r + g + b
          var c = -1
          for (i, q) in patterns.enumerated() {
            if (q.0 ? r > 170 : r < 80) && (q.1 ? g > 170 : g < 80) && (q.2 ? b > 170 : b < 80) { c = i; break }
          }
          let d = abs(b - Int(f[o])) + abs(g - Int(f[o + 1])) + abs(r - Int(f[o + 2]))
          for k in [c, d > 90 ? NC - 1 : -1] where k >= 0 {
            let j = k * OH + y
            if mn[j] > Int16(x) { mn[j] = Int16(x) }
            mx[j] = Int16(x); ct[j] += 1
          }
        }
        lum[y] = UInt8(ls / (3 * w))
      }
    }
    if let dp = ProcessInfo.processInfo.environment["CAPDUMP"], n == 0 || n % 30 == 0 {
      let ci = CIImage(cvPixelBuffer: pb); let ctx = CIContext()
      if let cg = ctx.createCGImage(ci, from: ci.extent) { let r = NSBitmapImageRep(cgImage: cg); try? r.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: "\(dp)-\(n).png")) }
    }
    var d = Data()
    var tt = CMTimeGetSeconds(CMSampleBufferGetPresentationTimeStamp(sb))
    d.append(Data(bytes: &tt, count: 8))
    for k in 0..<NC { for y in 0..<OH { var v = [mn[k * OH + y], mx[k * OH + y], ct[k * OH + y]]; d.append(Data(bytes: &v, count: 6)) } }
    d.append(contentsOf: lum)
    fh.write(d); n += 1
  }
}

func key(_ code: UInt16, _ ctrl: Bool) {
  let src = CGEventSource(stateID: .hidSystemState)
  for down in [true, false] {
    let e = CGEvent(keyboardEventSource: src, virtualKey: code, keyDown: down)!
    if ctrl { e.flags = [.maskControl, .maskSecondaryFn, .maskNumericPad] }
    e.post(tap: .cghidEventTap)
  }
}
func move(_ x: Double, _ y: Double) {
  CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: CGPoint(x: x, y: y), mouseButton: .left)!.post(tap: .cghidEventTap)
}
func run(_ act: String) async {
  let parts = act.split(separator: ":", maxSplits: 1).map(String.init)
  let arg = parts.count > 1 ? parts[1] : ""
  let nums = arg.split(separator: ",").compactMap { Double($0) }
  evlog += String(format: "%.6f %@\n", hostNow(), act)
  switch parts[0] {
  case "k": key(UInt16(nums[0]), true)
  case "kp": key(UInt16(nums[0]), false)
  case "ok":
    let p = Process(); p.launchPath = "/usr/bin/osascript"
    p.arguments = ["-e", "tell application \"System Events\" to key code \(arg) using control down"]; try? p.run(); p.waitUntilExit()
  case "sh":
    let p = Process(); p.launchPath = "/bin/sh"; p.arguments = ["-c", arg]; try? p.run()
  case "m": move(nums[0], nums[1])
  case "g":
    let steps = max(1, Int(nums[4] / 8))
    for i in 1...steps {
      let f = Double(i) / Double(steps)
      move(nums[0] + (nums[2] - nums[0]) * f, nums[1] + (nums[3] - nums[1]) * f)
      try? await Task.sleep(nanoseconds: 8_000_000)
    }
  case "d", "u":
    let e = CGEvent(mouseEventSource: CGEventSource(stateID: .hidSystemState), mouseType: parts[0] == "d" ? .leftMouseDown : .leftMouseUp, mouseCursorPosition: CGPoint(x: nums[0], y: nums[1]), mouseButton: .left)!
    e.setIntegerValueField(.mouseEventClickState, value: 1); e.post(tap: .cghidEventTap)
  case "dg":
    let steps = max(1, Int(nums[4] / 8))
    for i in 1...steps {
      let f = Double(i) / Double(steps)
      CGEvent(mouseEventSource: CGEventSource(stateID: .hidSystemState), mouseType: .leftMouseDragged, mouseCursorPosition: CGPoint(x: nums[0] + (nums[2] - nums[0]) * f, y: nums[1] + (nums[3] - nums[1]) * f), mouseButton: .left)!.post(tap: .cghidEventTap)
      try? await Task.sleep(nanoseconds: 8_000_000)
    }
  case "s": try? await Task.sleep(nanoseconds: UInt64(nums[0] * 1_000_000))
  case "t": FileManager.default.createFile(atPath: arg, contents: nil)
  default: print("bad action", act)
  }
  evlog += String(format: "%.6f end\n", hostNow())
}

let out = Out()
let sem = DispatchSemaphore(value: 0)
Task {
  do {
    let c = try await SCShareableContent.current
    let f = SCContentFilter(display: c.displays.first!, excludingWindows: [])
    let cfg = SCStreamConfiguration()
    cfg.pixelFormat = kCVPixelFormatType_32BGRA; cfg.sourceRect = src; cfg.width = OW; cfg.height = OH
    cfg.minimumFrameInterval = CMTime(value: 1, timescale: 180); cfg.queueDepth = 8; cfg.showsCursor = false
    let s = SCStream(filter: f, configuration: cfg, delegate: nil)
    try s.addStreamOutput(out, type: .screen, sampleHandlerQueue: DispatchQueue(label: "q"))
    try await s.startCapture()
    let start = Date()
    try await Task.sleep(nanoseconds: 400_000_000)
    for act in actions { await run(act) }
    let left = secs - Date().timeIntervalSince(start)
    if left > 0 { try await Task.sleep(nanoseconds: UInt64(left * 1e9)) }
    try await s.stopCapture()
  } catch { print("error", error) }
  sem.signal()
}
sem.wait()
try! evlog.write(toFile: outPath + ".ev", atomically: true, encoding: .utf8)
try! "\(OW) \(OH) \(NC) \(src.minX) \(src.minY) \(src.width) \(src.height)\n".write(toFile: outPath + ".hdr", atomically: true, encoding: .utf8)
print("frames", out.n)
