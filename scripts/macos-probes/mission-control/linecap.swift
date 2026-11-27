import AppKit
import ScreenCaptureKit
import CoreMedia

let a = CommandLine.arguments
let outPath = a[1]
let secs = Double(a[2])!
let rows = a[3].split(separator: ",").map { Int($0)! }
let cols = a[4].split(separator: ",").map { Int($0)! }
let actions = Array(a.dropFirst(5))
let W = 2560, H = 200
FileManager.default.createFile(atPath: outPath + ".bin", contents: nil)
let fh = FileHandle(forWritingAtPath: outPath + ".bin")!
var evlog = ""
func hostNow() -> Double { CMTimeGetSeconds(CMClockGetTime(CMClockGetHostTimeClock())) }

final class Out: NSObject, SCStreamOutput {
  func stream(_ s: SCStream, didOutputSampleBuffer sb: CMSampleBuffer, of t: SCStreamOutputType) {
    guard t == .screen, let atts = CMSampleBufferGetSampleAttachmentsArray(sb, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
          let st = atts.first?[.status] as? Int, st == SCFrameStatus.complete.rawValue,
          let pb = CVPixelBufferGetBaseAddress(pbLock(sb)) else { return }
    let bpr = CVPixelBufferGetBytesPerRow(cur!)
    let p = pb.assumingMemoryBound(to: UInt8.self)
    var tt = CMTimeGetSeconds(CMSampleBufferGetPresentationTimeStamp(sb))
    var b = [UInt8](repeating: 0, count: 8 + (rows.count * W + cols.count * H) * 3)
    withUnsafeBytes(of: &tt) { for i in 0..<8 { b[i] = $0[i] } }
    var i = 8
    for y in rows { for x in 0..<W { let o = y * bpr + x * 4; b[i] = p[o + 2]; b[i + 1] = p[o + 1]; b[i + 2] = p[o]; i += 3 } }
    for x in cols { for y in 0..<H { let o = y * bpr + x * 4; b[i] = p[o + 2]; b[i + 1] = p[o + 1]; b[i + 2] = p[o]; i += 3 } }
    CVPixelBufferUnlockBaseAddress(cur!, .readOnly)
    fh.write(Data(b))
  }
}
var cur: CVPixelBuffer?
func pbLock(_ sb: CMSampleBuffer) -> CVPixelBuffer {
  let pb = CMSampleBufferGetImageBuffer(sb)!
  CVPixelBufferLockBaseAddress(pb, .readOnly); cur = pb
  return pb
}

func key(_ code: UInt16) {
  let src = CGEventSource(stateID: .hidSystemState)
  for down in [true, false] {
    let e = CGEvent(keyboardEventSource: src, virtualKey: code, keyDown: down)!
    e.flags = [.maskControl, .maskSecondaryFn, .maskNumericPad]
    e.post(tap: .cghidEventTap)
  }
}
func mouse(_ type: CGEventType, _ x: Double, _ y: Double) {
  let e = CGEvent(mouseEventSource: CGEventSource(stateID: .hidSystemState), mouseType: type, mouseCursorPosition: CGPoint(x: x, y: y), mouseButton: .left)!
  e.setIntegerValueField(.mouseEventClickState, value: 1)
  e.post(tap: .cghidEventTap)
}
func run(_ act: String) async {
  let parts = act.split(separator: ":", maxSplits: 1).map(String.init)
  let nums = (parts.count > 1 ? parts[1] : "").split(separator: ",").compactMap { Double($0) }
  evlog += String(format: "%.6f %@\n", hostNow(), act)
  switch parts[0] {
  case "k": key(UInt16(nums[0]))
  case "m": mouse(.mouseMoved, nums[0], nums[1])
  case "d": mouse(.leftMouseDown, nums[0], nums[1])
  case "u": mouse(.leftMouseUp, nums[0], nums[1])
  case "s": try? await Task.sleep(nanoseconds: UInt64(nums[0] * 1_000_000))
  default: print("bad action", act)
  }
}

let out = Out()
let sem = DispatchSemaphore(value: 0)
Task {
  do {
    let c = try await SCShareableContent.current
    let f = SCContentFilter(display: c.displays.first!, excludingWindows: [])
    let cfg = SCStreamConfiguration()
    cfg.pixelFormat = kCVPixelFormatType_32BGRA
    cfg.sourceRect = CGRect(x: 0, y: 0, width: W, height: H)
    cfg.width = W; cfg.height = H
    cfg.minimumFrameInterval = CMTime(value: 1, timescale: 180)
    cfg.queueDepth = 8; cfg.showsCursor = false
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
  try? evlog.write(toFile: outPath + ".ev", atomically: true, encoding: .utf8)
  try? "\(rows.count) \(cols.count) \(W) \(H)\n".write(toFile: outPath + ".hdr", atomically: true, encoding: .utf8)
  sem.signal()
}
sem.wait()
