import AppKit

let dataDir = CommandLine.arguments[1]
let metaName = CommandLine.arguments[2]
let report = [1, 2, 3, 5, 8, 10, 15, 20, 30, 40, 60]
var cache: [String: NSBitmapImageRep] = [:]

func image(_ shot: String) -> NSBitmapImageRep {
    if let r = cache[shot] { return r }
    let r = NSBitmapImageRep(data: try! Data(contentsOf: URL(fileURLWithPath: "\(dataDir)/\(shot).png")))!
    precondition(r.bitsPerSample == 8 && r.pixelsWide == 2560)
    cache[shot] = r
    return r
}

func px(_ r: NSBitmapImageRep, _ x: Int, _ y: Int) -> Double {
    var p = [Int](repeating: 0, count: 4)
    r.getPixel(&p, atX: x, y: y)
    return Double(p[0] + p[1] + p[2]) / 3
}

func fit(_ ds: [Double], _ a: [Double], _ model: (Double, Double) -> Double) -> (Double, Double, Double) {
    var best = (0.0, 0.0, Double.infinity)
    for k in 5...800 {
        let s = Double(k) / 10
        let e = ds.map { model($0, s) }
        let amp = zip(a, e).map(*).reduce(0, +) / e.map { $0 * $0 }.reduce(0, +)
        let rms = sqrt(zip(a, e).map { pow($0 - amp * $1, 2) }.reduce(0, +) / Double(a.count))
        if rms < best.2 { best = (amp, s, rms) }
    }
    return best
}

var csv = "shot window state side d px alpha\n"
var summary = ""
for line in try! String(contentsOfFile: "\(dataDir)/\(metaName)", encoding: .utf8).split(separator: "\n") {
    let f = line.split(separator: " ").map(String.init)
    let (shot, win) = (f[0], f[1])
    let (x0, y0, w, h) = (Int(f[2])!, Int(f[3])!, Int(f[4])!, Int(f[5])!)
    let state = f[6] == "key=true" ? "active" : "inactive"
    let r = image(shot)
    let bgPoint = f.first { $0.hasPrefix("bg=") }.map { $0.dropFirst(3).split(separator: ",").map { Int($0)! } } ?? [2, 720]
    let bg = px(r, bgPoint[0], bgPoint[1])
    summary += "\n\(shot) \(win) \(state) frame \(x0),\(y0) \(w)x\(h) bg \(String(format: "%.1f", bg))\n"
    for side in ["top", "bottom", "left", "right"] {
        func sample(_ d: Int) -> Double {
            var s = 0.0
            for t in -20...20 {
                switch side {
                case "top": s += px(r, x0 + w / 2 + t, y0 - d)
                case "bottom": s += px(r, x0 + w / 2 + t, y0 + h - 1 + d)
                case "left": s += px(r, x0 - d, y0 + h / 2 + t)
                default: s += px(r, x0 + w - 1 + d, y0 + h / 2 + t)
                }
            }
            return s / 41
        }
        let values = (-30...100).map { ($0, sample($0)) }
        var alphas: [Int: Double] = [:]
        for (d, v) in values {
            let a = bg > 8 ? 1 - v / bg : -1
            alphas[d] = a
            csv += "\(shot) \(win) \(state) \(side) \(d) \(String(format: "%.2f", v)) \(String(format: "%.4f", a))\n"
        }
        let rim = (-2...3).map { String(format: "%.0f", sample($0)) }.joined(separator: ",")
        if bg <= 8 {
            summary += "  \(side.padding(toLength: 6, withPad: " ", startingAt: 0)) rim[-2..3] \(rim) | px " + report.map { String(format: "%.1f", sample($0 + 1)) }.joined(separator: " ") + "\n"
            continue
        }
        let ds = (1...60).map(Double.init)
        let as_ = (1...60).map { alphas[$0 + 1]! }
        let e = fit(ds, as_) { d, s in exp(-d / s) }
        let g = fit(ds, as_) { d, s in exp(-d * d / (2 * s * s)) }
        let extent = (1...99).first { d in (d...99).allSatisfy { alphas[$0 + 1]! < 0.005 } } ?? -1
        summary += "  \(side.padding(toLength: 6, withPad: " ", startingAt: 0)) rim[-2..3] \(rim) | a " + report.map { String(format: "%.3f", alphas[$0 + 1]!) }.joined(separator: " ")
        summary += String(format: " | exp %.3f/%.1f rms %.4f | gauss %.3f/%.1f rms %.4f | <0.005 at %d\n", e.0, e.1, e.2, g.0, g.1, g.2, extent)
    }
}
try! csv.write(toFile: "\(dataDir)/profiles-\(metaName)", atomically: true, encoding: .utf8)
print(summary)
