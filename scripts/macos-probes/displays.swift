import AppKit
for (i, s) in NSScreen.screens.enumerated() {
    print("screen", i, s.localizedName)
    print(" frame", s.frame, "visible", s.visibleFrame)
    print(" scale", s.backingScaleFactor, "safeTop", s.safeAreaInsets.top, "maxFPS", s.maximumFramesPerSecond)
    if let id = s.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? CGDirectDisplayID {
        let m = CGDisplayCopyDisplayMode(id)!
        print(" mode", m.width, "x", m.height, "px", m.pixelWidth, "x", m.pixelHeight, "hz", m.refreshRate)
        let opts = [kCGDisplayShowDuplicateLowResolutionModes: true] as CFDictionary
        let all = CGDisplayCopyAllDisplayModes(id, opts) as! [CGDisplayMode]
        for d in all where d.isUsableForDesktopGUI() { print("  avail", d.width, "x", d.height, "px", d.pixelWidth, "x", d.pixelHeight, "hz", d.refreshRate) }
    }
}
