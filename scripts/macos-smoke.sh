#!/bin/sh
# Builds every recipe of crates/bk/examples, opens each as an app bundle for a few seconds
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
    names=$(for file in crates/bk/examples/*.rs; do basename "$file" .rs; done)
fi
builds=
for name in $names; do builds="$builds --example $name"; done
# shellcheck disable=SC2086
cargo build -p bk $builds
examples=${CARGO_TARGET_DIR:-$root/target}/debug/examples
# shellcheck source-path=SCRIPTDIR source=macos-app.sh
. "$root/scripts/macos-app.sh"

# Prints nothing when the recipe passes, or why it failed.
smoke() {
    [ -x "$examples/$1" ] || { echo "no such example"; return; }
    app=$work/$1.app
    bundle "$app" "$examples/$1" "local.barrierkit.smoke.$1"
    open -n --stderr "$work/$1.log" "$app"
    pid=
    tries=0
    while [ "$tries" -lt 40 ]; do
        pid=$(pgrep -n -f "$app/Contents/MacOS/$1" || true)
        [ -n "$pid" ] && [ "$(window_of "$pid")" -ne 0 ] && break
        tries=$((tries + 1))
        sleep 0.25
    done
    if [ -z "$pid" ] || [ "$(window_of "$pid")" -eq 0 ]; then
        [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
        echo "no window"
        return
    fi
    sleep "$seconds"
    if ! kill -0 "$pid" 2>/dev/null; then
        echo "exited"
        return
    fi
    if [ "$(window_of "$pid")" -eq 0 ]; then
        kill "$pid" 2>/dev/null || true
        echo "lost its window"
        return
    fi
    [ -z "${CAPTURE_DIR:-}" ] || { mkdir -p "$CAPTURE_DIR" && screencapture -l "$(window_of "$pid")" -o "$CAPTURE_DIR/$1.png"; }
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
