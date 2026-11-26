import AppKit
import ScreenCaptureKit
import CoreMedia
import CoreImage

let a = CommandLine.arguments
let prefix = a[1], secs = Double(a[2])!, trigger = a[3]
let W = a.count > 5 ? Int(a[4])! : 640, H = a.count > 5 ? Int(a[5])! : 360
var fired = 0.0
let ctx = CIContext()
final class Out: NSObject, SCStreamOutput {
  func stream(_ s: SCStream, didOutputSampleBuffer sb: CMSampleBuffer, of t: SCStreamOutputType) {
    guard fired > 0, t == .screen, let atts = CMSampleBufferGetSampleAttachmentsArray(sb, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
          let st = atts.first?[.status] as? Int, st == SCFrameStatus.complete.rawValue, let pb = CMSampleBufferGetImageBuffer(sb) else { return }
    let ms = (CMTimeGetSeconds(CMSampleBufferGetPresentationTimeStamp(sb)) - fired) * 1000
    let ci = CIImage(cvPixelBuffer: pb)
    guard let cg = ctx.createCGImage(ci, from: ci.extent) else { return }
    try? NSBitmapImageRep(cgImage: cg).representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: String(format: "%@-%04d.png", prefix, Int(ms.rounded()) + 1000)))
  }
}
let out = Out()
let sem = DispatchSemaphore(value: 0)
Task {
  do {
    let c = try await SCShareableContent.current
    let cfg = SCStreamConfiguration()
    cfg.pixelFormat = kCVPixelFormatType_32BGRA; cfg.width = W; cfg.height = H
    cfg.minimumFrameInterval = CMTime(value: 1, timescale: 180); cfg.queueDepth = 8; cfg.showsCursor = false
    let s = SCStream(filter: SCContentFilter(display: c.displays.first!, excludingWindows: []), configuration: cfg, delegate: nil)
    try s.addStreamOutput(out, type: .screen, sampleHandlerQueue: DispatchQueue(label: "q"))
    try await s.startCapture()
    try await Task.sleep(nanoseconds: 300_000_000)
    fired = CMTimeGetSeconds(CMClockGetTime(CMClockGetHostTimeClock()))
    FileManager.default.createFile(atPath: trigger, contents: nil)
    try await Task.sleep(nanoseconds: UInt64(secs * 1e9))
    try await s.stopCapture()
  } catch { print("error", error) }
  sem.signal()
}
sem.wait()
