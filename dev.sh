#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)

require_devshell() {
    [ "${BARRIERKIT_DEV_SHELL:-}" = "1" ] ||
        fail 2 "not inside the dev shell; run 'nix develop' first"
}

require() {
    command -v "$1" >/dev/null 2>&1 || fail 2 "'$1' not found"
}

fail() {
    _code=$1
    shift
    printf 'dev.sh: %s\n' "$*" >&2
    exit "$_code"
}

help() {
    cat <<'EOF'
Usage: ./dev.sh <command>

Commands:
  app <name> [args]       run tengen-app-show, marea or ondina against the current WAYLAND_DISPLAY
  check                   run fmt, clippy, tests and kit-specific checks
  help                    show this message

All commands require 'nix develop'.
EOF
}

app() {
    require_devshell
    require cargo
    _name=${1:-}
    case "$_name" in
        tengen-app-show | marea | ondina) shift ;;
        *) fail 1 "usage: ./dev.sh app tengen-app-show|marea|ondina [args]" ;;
    esac
    RUST_LOG=${RUST_LOG:-info} cargo run -p "$_name" -- "$@"
}

check() {
    require_devshell
    require cargo
    require cargo-fmt
    require cargo-clippy
    cargo fmt --all -- --check
    cargo clippy -p barrier-kit --lib -- -D warnings
    cargo clippy --workspace --all-targets --features barrier-kit/shell -- -D warnings
    "$script_dir/scripts/check-design-tokens.sh"
    "$script_dir/scripts/check-client-only-deps.sh"
    cargo test --workspace --features barrier-kit/shell
}

case "${1:-}" in
    app)
        shift
        app "$@"
        ;;
    check)
        check
        ;;
    help|--help|-h|'')
        help
        ;;
    *)
        fail 1 "unknown command: $1"
        ;;
esac
