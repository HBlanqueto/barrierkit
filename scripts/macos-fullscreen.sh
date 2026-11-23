#!/bin/sh
# Puts the kit's windows and the same windows of a SwiftUI app into full screen and out of it,
# and fails when the kit does not do what SwiftUI does.
# Usage: scripts/macos-fullscreen.sh   (CAPTURE_DIR: a folder that receives a PNG of each window in full screen)
# Needs an unlocked session and the Accessibility permission for System Events and the pointer.
set -eu

[ "$(uname -s)" = Darwin ] || { echo "macos-fullscreen: macOS only" >&2; exit 2; }

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
work=$(mktemp -d)
cleanup() { pkill -f "$work/" 2>/dev/null || true; rm -rf "$work"; }
trap cleanup EXIT
trap 'exit 130' INT TERM

cd "$root"
# shellcheck source-path=SCRIPTDIR source=macos-app.sh
. "$root/scripts/macos-app.sh"
cargo build -p bk --example full_screen
kit=${CARGO_TARGET_DIR:-$root/target}/debug/examples/full_screen
swiftc -parse-as-library -O scripts/fullscreen-reference.swift -o "$work/swift-title-bar"
swiftc -parse-as-library -O -DHIDDEN scripts/fullscreen-reference.swift -o "$work/swift-hidden-title-bar"
swiftc -parse-as-library -O -DPLAIN scripts/fullscreen-reference.swift -o "$work/swift-plain"
swiftc -O scripts/click.swift -o "$work/click"
swiftc -O scripts/window-trace.swift -o "$work/window-trace"

screen=$(osascript -e 'tell application "Finder" to get bounds of window of desktop' | awk -F', ' '{print $3 "x" $4}')

# Opens <binary> <arg> as <name>.app and prints its pid once its window shows.
launch() {
    app=$work/$1.app
    bundle "$app" "$2" "local.barrierkit.fullscreen.$1"
    if [ -n "${3:-}" ]; then
        open -n --stderr "$work/$1.log" "$app" --args "$3"
    else
        open -n --stderr "$work/$1.log" "$app"
    fi
    pid=
    for _ in $(seq 40); do
        pid=$(pgrep -n -f "$app/Contents/MacOS/$1" || true)
        [ -n "$pid" ] && [ -n "$(ax "$pid" 'size of window 1')" ] && break
        sleep 0.25
    done
    echo "$pid"
}

# Runs an AppleScript expression against the first window of a process; prints "WxH" for a size.
ax() {
    osascript -e "tell application \"System Events\" to tell (first process whose unix id is $1) to $2" 2>/dev/null |
        awk -F', ' 'NF == 2 { print $1 "x" $2; next } { print }' || true
}

full_screen() {
    osascript -e "tell application \"System Events\" to tell (first process whose unix id is $1) to set value of attribute \"AXFullScreen\" of window 1 to $2" >/dev/null 2>&1 || true
}

# The kit's own green light: 72 and 26 from the window's corner, as the Titlebar draws it.
press_kit_green() {
    position=$(ax "$1" 'position of window 1')
    "$work/click" "$((${position%x*} + 72))" "$((${position#*x} + 26))"
}

# Prints "<size in window> <size in full screen> <size back> told|silent" for one window; told: the app learned it was in full screen.
exercise() {
    name=$1 binary=$2 style=$3 kind=$4
    pid=$(launch "$name" "$binary" "$5")
    [ -n "$pid" ] || { echo "no-window - - -"; return; }
    osascript -e "tell application \"System Events\" to set frontmost of (first process whose unix id is $pid) to true" >/dev/null 2>&1
    sleep 0.5
    before=$(ax "$pid" 'size of window 1')
    "$work/window-trace" "$pid" 3.5 > "$work/$name.enter" &
    if [ "$kind" = kit-plain ]; then press_kit_green "$pid"; else full_screen "$pid" true; fi
    wait
    during=$(ax "$pid" 'size of window 1')
    [ -z "${CAPTURE_DIR:-}" ] || { mkdir -p "$CAPTURE_DIR" && screencapture -x "$CAPTURE_DIR/$name.png"; }
    if [ "$kind" = kit-plain ]; then
        # The "Toggle Full Screen" button, centred under the 52 high bar.
        h=${during#*x}
        w=${during%x*}
        "$work/click" "$((w / 2))" "$(((h + 52) / 2 + 14))"
    else
        full_screen "$pid" false
    fi
    sleep 3
    echo "  $name: $(wc -l < "$work/$name.enter" | tr -d ' ') bounds seen entering full screen" >&2
    after=$(ax "$pid" 'size of window 1')
    last=$(grep -q 'full-screen: true' "$work/$name.log" 2>/dev/null && echo told || echo silent)
    kill "$pid" 2>/dev/null || true
    echo "${before:--} ${during:--} ${after:--} ${last:--}"
}

failed=0
# shellcheck disable=SC2086
for style in title-bar hidden-title-bar plain; do
    case $style in plain) kind='kit-plain' ;; *) kind=kit ;; esac
    kit_result=$(exercise "kit-$style" "$kit" "$style" "$kind" "$style")
    swift_result=$(exercise "swift-$style" "$work/swift-$style" "$style" swift "")
    echo "$style"
    echo "  kit    (window, full screen, back, told): $kit_result"
    echo "  swiftui (window, full screen, back, told): $swift_result"
    set -- $kit_result
    kit_enters=$([ "$2" = "$screen" ] && echo yes || echo no)
    kit_back=$([ "$3" = "$1" ] && echo yes || echo no)
    kit_told=$([ "$4" = told ] && echo yes || echo no)
    set -- $swift_result
    swift_enters=$([ "$2" = "$screen" ] && echo yes || echo no)
    swift_back=$([ "$3" = "$1" ] && echo yes || echo no)
    # Where SwiftUI cannot enter full screen (a plain window), the kit may still do it.
    if { [ "$kit_enters" = "$swift_enters" ] || [ "$kit_enters" = yes ]; } && [ "$kit_back" = yes ] && { [ "$kit_enters" = no ] || [ "$kit_told" = yes ]; }; then
        echo "  PASS (SwiftUI enters: $swift_enters, kit enters: $kit_enters)"
    else
        echo "  FAIL: SwiftUI enters $swift_enters and returns $swift_back; kit enters $kit_enters, returns $kit_back, tells the app $kit_told"
        failed=$((failed + 1))
    fi
done

echo "$failed failed"
[ "$failed" -eq 0 ]
