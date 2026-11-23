#!/bin/sh
# Builds every recipe of agents/recipes, opens each as an app bundle for a few seconds
# and checks that a window appears and the process stays alive.
# Usage: scripts/macos-smoke.sh [recipe...]   (SMOKE_SECONDS: how long each stays open, default 3;
# CAPTURE_DIR: a folder that receives a PNG of each recipe's window)
set -eu

[ "$(uname -s)" = Darwin ] || { echo "macos-smoke: macOS only" >&2; exit 2; }

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
seconds=${SMOKE_SECONDS:-3}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
trap 'exit 130' INT TERM

cd "$root"
if [ "$#" -gt 0 ]; then
    names=$*
else
    names=$(for file in agents/recipes/*.rs; do basename "$file" .rs; done)
fi
builds=
for name in $names; do builds="$builds --example $name"; done
# shellcheck disable=SC2086
cargo build -p bk $builds
examples=${CARGO_TARGET_DIR:-$root/target}/debug/examples

window_id() {
    osascript -l JavaScript -e "
        ObjC.import('CoreGraphics');
        var all = ObjC.deepUnwrap(ObjC.castRefToObject(
            \$.CGWindowListCopyWindowInfo(\$.kCGWindowListOptionOnScreenOnly, 0)));
        all.filter(function (w) {
            return w.kCGWindowOwnerPID == $1 && w.kCGWindowLayer == 0;
        })[0].kCGWindowNumber" 2>/dev/null
}

window_count() {
    osascript -l JavaScript -e "
        ObjC.import('CoreGraphics');
        var all = ObjC.deepUnwrap(ObjC.castRefToObject(
            \$.CGWindowListCopyWindowInfo(\$.kCGWindowListOptionOnScreenOnly, 0)));
        all.filter(function (w) {
            return w.kCGWindowOwnerPID == $1 && w.kCGWindowLayer == 0;
        }).length" 2>/dev/null || echo 0
}

bundle() {
    app=$work/$1.app
    mkdir -p "$app/Contents/MacOS"
    cp "$examples/$1" "$app/Contents/MacOS/$1"
    cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>$1</string>
<key>CFBundleIdentifier</key><string>local.barrierkit.smoke.$1</string>
<key>CFBundleName</key><string>$1</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
    codesign --force --sign - "$app" >/dev/null 2>&1
}

# Prints nothing when the recipe passes, or why it failed.
smoke() {
    [ -x "$examples/$1" ] || { echo "no such example"; return; }
    bundle "$1"
    open -n --stderr "$work/$1.log" "$app"
    pid=
    tries=0
    while [ "$tries" -lt 40 ]; do
        pid=$(pgrep -n -f "$app/Contents/MacOS/$1" || true)
        [ -n "$pid" ] && [ "$(window_count "$pid")" -gt 0 ] && break
        tries=$((tries + 1))
        sleep 0.25
    done
    if [ -z "$pid" ] || [ "$(window_count "$pid")" -eq 0 ]; then
        [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
        echo "no window"
        return
    fi
    sleep "$seconds"
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "exited"
        return
    fi
    if [ "$(window_count "$pid")" -eq 0 ]; then
        kill "$pid" 2>/dev/null || true
        echo "lost its window"
        return
    fi
    [ -z "${CAPTURE_DIR:-}" ] || { mkdir -p "$CAPTURE_DIR" && screencapture -l "$(window_id "$pid")" -o "$CAPTURE_DIR/$1.png"; }
    kill "$pid"
    while kill -0 "$pid" 2>/dev/null; do sleep 0.1; done
}

failed=0
for name in $names; do
    reason=$(smoke "$name")
    if [ -z "$reason" ]; then
        echo "PASS $name"
    else
        echo "FAIL $name: $reason"
        grep -iE "panic|error" "$work/$name.log" 2>/dev/null | head -3 | sed 's/^/     /' || true
        failed=$((failed + 1))
        killall UserNotificationCenter 2>/dev/null || true
    fi
done

echo "$failed failed"
[ "$failed" -eq 0 ]
