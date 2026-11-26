import AppKit
import ScreenCaptureKit
import CoreMedia

// cap OUT secs delay "sh command" x,y [x,y ...]: 180 fps full-display stream at 1/4 size, sRGB;
// per frame: t and the 11x11 (44x44 pt) mean RGB at each point; runs the command after `delay` s and logs its host time
let a = CommandLine.arguments
let outPath = a[1], secs = Double(a[2])!, delay = Double(a[3])!, cmd = a[4]
let pts = a.dropFirst(5).map { $0.split(separator: ",").map { Int($0)! / 4 } }
let OW = 640, OH = 360
var lines = ""
let lock = NSLock()
func hostNow() -> Double { CMTimeGetSeconds(CMClockGetTime(CMClockGetHostTimeClock())) }

final class Out: NSObject, SCStreamOutput {
  func stream(_ s: SCStream, didOutputSampleBuffer sb: CMSampleBuffer, of t: SCStreamOutputType) {
    guard t == .screen, let atts = CMSampleBufferGetSampleAttachmentsArray(sb, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
          let st = atts.first?[.status] as? Int, st == SCFrameStatus.complete.rawValue,
          let pb = CMSampleBufferGetImageBuffer(sb) else { return }
    CVPixelBufferLockBaseAddress(pb, .readOnly); defer { CVPixelBufferUnlockBaseAddress(pb, .readOnly) }
    let bpr = CVPixelBufferGetBytesPerRow(pb)
    let p = CVPixelBufferGetBaseAddress(pb)!.assumingMemoryBound(to: UInt8.self)
    var l = String(format: "%.6f", CMTimeGetSeconds(CMSampleBufferGetPresentationTimeStamp(sb)))
    for q in pts {
      var s = [0.0, 0.0, 0.0]
      for y in q[1]-5...q[1]+5 { for x in q[0]-5...q[0]+5 { let o = y * bpr + x * 4; s[0] += Double(p[o + 2]); s[1] += Double(p[o + 1]); s[2] += Double(p[o]) } }
      l += String(format: " %.1f %.1f %.1f", s[0] / 121, s[1] / 121, s[2] / 121)
    }
    lock.lock(); lines += l + "\n"; lock.unlock()
  }
}

let out = Out()
let sem = DispatchSemaphore(value: 0)
var ev = ""
Task {
  do {
    let c = try await SCShareableContent.current
    let cfg = SCStreamConfiguration()
    cfg.pixelFormat = kCVPixelFormatType_32BGRA; cfg.width = OW; cfg.height = OH
    cfg.colorSpaceName = CGColorSpace.sRGB
    cfg.minimumFrameInterval = CMTime(value: 1, timescale: 180); cfg.queueDepth = 8; cfg.showsCursor = false
    let s = SCStream(filter: SCContentFilter(display: c.displays.first!, excludingWindows: []), configuration: cfg, delegate: nil)
    try s.addStreamOutput(out, type: .screen, sampleHandlerQueue: DispatchQueue(label: "q"))
    try await s.startCapture()
    try await Task.sleep(nanoseconds: UInt64(delay * 1e9))
    ev = String(format: "%.6f start\n", hostNow())
    let p = Process(); p.launchPath = "/bin/sh"; p.arguments = ["-c", cmd]; try p.run(); p.waitUntilExit()
    ev += String(format: "%.6f end\n", hostNow())
    try await Task.sleep(nanoseconds: UInt64((secs - delay) * 1e9))
    try await s.stopCapture()
  } catch { print("error", error) }
  sem.signal()
}
sem.wait()
try! (ev + lines).write(toFile: outPath, atomically: true, encoding: .utf8)
