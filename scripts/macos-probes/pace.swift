import AppKit
import CoreVideo
var link: CVDisplayLink?
CVDisplayLinkCreateWithCGDisplay(CGMainDisplayID(), &link)
var times = [UInt64]()
let lock = NSLock()
CVDisplayLinkSetOutputHandler(link!) { _, _, out, _, _ in
  lock.lock(); times.append(out.pointee.hostTime); lock.unlock(); return kCVReturnSuccess }
CVDisplayLinkStart(link!)
Thread.sleep(forTimeInterval: 3)
CVDisplayLinkStop(link!)
var tb = mach_timebase_info_data_t(); mach_timebase_info(&tb)
let ms = zip(times.dropFirst(), times).map { Double($0 - $1) * Double(tb.numer) / Double(tb.denom) / 1e6 }.sorted()
print("frames", times.count, "nominal", CVDisplayLinkGetNominalOutputVideoRefreshPeriod(link!).timeValue, "/", CVDisplayLinkGetNominalOutputVideoRefreshPeriod(link!).timeScale)
print("interval ms min", ms.first!, "median", ms[ms.count/2], "max", ms.last!)
