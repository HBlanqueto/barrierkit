# shellcheck shell=sh
# Sourced by the scripts that drive the screen; scripts/appearance-matrix.py mirrors it.
lock=/tmp/barrierkit-screen.lock
holding=

take_screen_lock() {
    waiting=
    until mkdir "$lock" 2>/dev/null; do
        holder=$(cat "$lock/pid" 2>/dev/null || true)
        # An empty pid file means the owner is between mkdir and writing it: still held.
        if [ -n "$holder" ] && ! kill -0 "$holder" 2>/dev/null; then
            rm -rf "$lock"
            continue
        fi
        [ -n "$waiting" ] || echo "waiting for $lock held by PID ${holder:-unknown}" >&2
        waiting=1
        sleep 2
    done
    holding=1
    echo $$ > "$lock/pid"
}

release_screen_lock() {
    [ -z "$holding" ] || rm -rf "$lock"
}
