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
let total = d(fr[0].1, fr.last!.1)
print("frames", fr.count, "total-diff", total)
var prev = fr[0].1; var last = 0.0
for (t, s) in fr { let step = d(prev, s)
  if step > 0.3 { print(String(format: "t=%.4f dt=%.2fms progress=%.3f", t - t0, (t - last) * 1000, d(fr[0].1, s) / max(total, 1))); last = t }
  prev = s }
