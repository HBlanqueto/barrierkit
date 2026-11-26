import AppKit
import ScreenCaptureKit
import CoreMedia
let key = Int(CommandLine.arguments[1])!
final class Out: NSObject, SCStreamOutput {
  var frames = [(Double, [Float])]()
  func stream(_ s: SCStream, didOutputSampleBuffer sb: CMSampleBuffer, of t: SCStreamOutputType) {
    guard t == .screen, let atts = CMSampleBufferGetSampleAttachmentsArray(sb, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
          let st = atts.first?[.status] as? Int, st == SCFrameStatus.complete.rawValue,
          let pb = CMSampleBufferGetImageBuffer(sb) else { return }
    CVPixelBufferLockBaseAddress(pb, .readOnly); defer { CVPixelBufferUnlockBaseAddress(pb, .readOnly) }
    let w = CVPixelBufferGetWidth(pb), h = CVPixelBufferGetHeight(pb), bpr = CVPixelBufferGetBytesPerRow(pb)
    let p = CVPixelBufferGetBaseAddress(pb)!.assumingMemoryBound(to: UInt8.self)
    var v = [Float](); v.reserveCapacity(64*36)
    for gy in 0..<36 { for gx in 0..<64 { let o = (gy*h/36)*bpr + (gx*w/64)*4; v.append(Float(p[o]) + Float(p[o+1]) + Float(p[o+2])) } }
    frames.append((CMTimeGetSeconds(CMSampleBufferGetPresentationTimeStamp(sb)), v))
  }
}
let out = Out()
let sem = DispatchSemaphore(value: 0)
Task {
  let c = try await SCShareableContent.current
  let disp = c.displays.first!
  let f = SCContentFilter(display: disp, excludingWindows: [])
  let cfg = SCStreamConfiguration()
  cfg.width = 1280; cfg.height = 720; cfg.minimumFrameInterval = CMTime(value: 1, timescale: 180); cfg.queueDepth = 8; cfg.showsCursor = false
  let s = SCStream(filter: f, configuration: cfg, delegate: nil)
  try s.addStreamOutput(out, type: .screen, sampleHandlerQueue: DispatchQueue(label: "q"))
  try await s.startCapture()
  try await Task.sleep(nanoseconds: 500_000_000)
  let p = Process(); p.launchPath = "/usr/bin/osascript"
  p.arguments = ["-e", "tell application \"System Events\" to key code \(key) using control down"]; try p.run()
  try await Task.sleep(nanoseconds: 2_000_000_000)
  try await s.stopCapture()
  sem.signal()
}
sem.wait()
let fr = out.frames; let t0 = fr[0].0
func d(_ a: [Float], _ b: [Float]) -> Float { zip(a, b).map { abs($0 - $1) }.reduce(0, +) / Float(a.count) }
let end = fr.last!.1
let total = d(fr[0].1, end)
var ev = [(Double, Float)]()
var prev = fr[0].1
for (t, s) in fr { if d(prev, s) > 0.3 { ev.append((t - t0, 1 - d(s, end) / total)) }; prev = s }
let a = ev.first!.0, b = ev.last!.0
print(String(format: "start %.4f end %.4f duration %.0f ms, %d changed frames", a, b, (b - a) * 1000, ev.count))
for f in [0.1, 0.25, 0.5, 0.75, 0.9, 0.97] { if let e = ev.first(where: { $0.1 >= Float(f) }) { print(String(format: "progress>=%.2f at +%.0f ms", f, (e.0 - a) * 1000)) } }
var gaps = 0; for i in 1..<ev.count where ev[i].0 - ev[i-1].0 > 0.0075 { gaps += 1 }
print("gaps over 7.5ms:", gaps)
