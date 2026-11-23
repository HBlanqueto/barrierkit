# Sourced by the macOS scripts.

# bundle <dir/name.app> <binary> <bundle id>: wraps the binary in an ad-hoc signed app named <name>.
bundle() {
    set -- "$1" "$2" "$3" "$(basename "$1" .app)"
    mkdir -p "$1/Contents/MacOS"
    cp "$2" "$1/Contents/MacOS/$4"
    cat > "$1/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>$4</string>
<key>CFBundleIdentifier</key><string>$3</string>
<key>CFBundleName</key><string>$4</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
    codesign --force --sign - "$1" >/dev/null 2>&1
}

# window_of <pid>: prints the number of the first normal window of the process, or 0.
window_of() {
    osascript -l JavaScript -e "
        ObjC.import('CoreGraphics');
        var all = ObjC.deepUnwrap(ObjC.castRefToObject(
            \$.CGWindowListCopyWindowInfo(\$.kCGWindowListOptionOnScreenOnly, 0)));
        var mine = all.filter(function (w) {
            return w.kCGWindowOwnerPID == $1 && w.kCGWindowLayer == 0; });
        mine.length ? mine[0].kCGWindowNumber : 0" 2>/dev/null || echo 0
}
