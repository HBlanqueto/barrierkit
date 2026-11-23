#!/bin/sh
# Captures the SwiftUI side of a twin: its window at 1x, through the screenshot's colour profile.
# Usage: scripts/twin-reference.sh <notes|tasks|form|controls|standalone> <light|dark> <out.png>
# Needs ~/src/probes/<probe>.swift (swiftc builds it), an unlocked session and the screen lock.
set -eu

[ "$#" -eq 3 ] || { sed -n '2,4p' "$0" >&2; exit 2; }
twin=$1 scheme=$2 out=$3
case "$twin" in
    notes) probe=csdtwin ;;
    tasks) probe=listtwin ;;
    form) probe=formtwin ;;
    controls) probe=controlstwin ;;
    standalone) probe=standalone ;;
    *) echo "twin-reference: unknown twin $twin" >&2; exit 2 ;;
esac
case "$scheme" in light) args= ;; dark) args=dark ;; *) echo "twin-reference: light or dark" >&2; exit 2 ;; esac

probes=${BARRIERKIT_PROBES:-$HOME/src/probes}
# shellcheck source=screen-lock.sh
. "$(dirname -- "$0")/screen-lock.sh"
work=$(mktemp -d)
cleanup() { pkill -f "$work/$probe.app/" 2>/dev/null || true; release_screen_lock; rm -rf "$work"; }
trap cleanup EXIT
trap 'exit 130' INT TERM
take_screen_lock

[ -x "$probes/$probe" ] || swiftc -O "$probes/$probe.swift" -o "$probes/$probe"
app=$work/$probe.app
mkdir -p "$app/Contents/MacOS"
cp "$probes/$probe" "$app/Contents/MacOS/$probe"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>$probe</string>
<key>CFBundleIdentifier</key><string>local.barrierkit.twin.$probe</string>
<key>CFBundleName</key><string>$probe</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
codesign --force --sign - "$app" >/dev/null 2>&1
# shellcheck disable=SC2086
open -n "$app" --args $args

window_of() {
    osascript -l JavaScript -e "
        ObjC.import('CoreGraphics');
        var all = ObjC.deepUnwrap(ObjC.castRefToObject(
            \$.CGWindowListCopyWindowInfo(\$.kCGWindowListOptionOnScreenOnly, 0)));
        var mine = all.filter(function (w) {
            return w.kCGWindowOwnerPID == $1 && w.kCGWindowLayer == 0; });
        mine.length ? mine[0].kCGWindowNumber : 0" 2>/dev/null || echo 0
}

id=0
tries=0
while [ "$id" -eq 0 ] && [ "$tries" -lt 60 ]; do
    pid=$(pgrep -n -f "$app/Contents/MacOS/$probe" || true)
    [ -n "$pid" ] && id=$(window_of "$pid")
    tries=$((tries + 1))
    sleep 0.25
done
[ "$id" -ne 0 ] || { echo "twin-reference: no window for $probe" >&2; exit 1; }
# The window draws its active look once the app has settled as the key window.
sleep 2
raw=$work/raw.png
screencapture -x -o -l "$id" "$raw"
sips -m "/System/Library/ColorSync/Profiles/sRGB Profile.icc" "$raw" --out "$out" >/dev/null
