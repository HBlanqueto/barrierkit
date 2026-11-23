#!/bin/sh
# Captures the SwiftUI side of a twin: its window at 1x, through the screenshot's colour profile.
# Usage: scripts/twin-reference.sh <notes|tasks|form|controls|standalone> <light|dark> <out.png>
# Needs ~/src/probes/<probe>.swift (swiftc builds it), and an unlocked session.
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
[ -d "$probes" ] || { echo "twin-reference: $probes not found; set BARRIERKIT_PROBES to the directory of the SwiftUI probes" >&2; exit 2; }
work=$(mktemp -d)
cleanup() { pkill -f "$work/$probe.app/" 2>/dev/null || true; rm -rf "$work"; }
trap cleanup EXIT
trap 'exit 130' INT TERM

# shellcheck source-path=SCRIPTDIR source=macos-app.sh
. "$(dirname -- "$0")/macos-app.sh"
[ -x "$probes/$probe" ] || swiftc -O "$probes/$probe.swift" -o "$probes/$probe"
app=$work/$probe.app
bundle "$app" "$probes/$probe" "local.barrierkit.twin.$probe"
# shellcheck disable=SC2086
open -n "$app" --args $args

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
