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
  app <name> [args]       run tengen-app-show, marea, ondina or trazo against the current WAYLAND_DISPLAY,
                          with MALAYSIA_CONFIG_DIR and MALAYSIA_DATA_DIR under $XDG_RUNTIME_DIR/barrierkit
  check                   run fmt, clippy, tests and kit-specific checks
  tracy <name> [args]     run an app in release with the Tracy layer and the same dirs; connect with 'tracy'
  help                    show this message

All commands require 'nix develop'.
EOF
}

require_app() {
    case "$2" in
        tengen-app-show | marea | ondina | trazo) ;;
        *) fail 1 "usage: ./dev.sh $1 tengen-app-show|marea|ondina|trazo [args]" ;;
    esac
}

scratch_dirs() {
    _scratch=${XDG_RUNTIME_DIR:-$script_dir/target}/barrierkit
    export MALAYSIA_CONFIG_DIR="${MALAYSIA_CONFIG_DIR:-$_scratch/config}"
    export MALAYSIA_DATA_DIR="${MALAYSIA_DATA_DIR:-$_scratch/data}"
    mkdir -p "$MALAYSIA_CONFIG_DIR" "$MALAYSIA_DATA_DIR"
}

app() {
    require_devshell
    require cargo
    _name=${1:-}
    require_app app "$_name"
    shift
    scratch_dirs
    RUST_LOG=${RUST_LOG:-info} cargo run -p "$_name" -- "$@"
}

tracy() {
    require_devshell
    require cargo
    _name=${1:-}
    require_app tracy "$_name"
    shift
    scratch_dirs
    RUST_LOG=${RUST_LOG:-info} cargo run --release --features barrier-kit/tracy -p "$_name" -- "$@"
}

check() {
    require_devshell
    require cargo
    require cargo-fmt
    require cargo-clippy
    cargo fmt --all -- --check
    cargo clippy -p barrier-kit --lib -- -D warnings
    cargo clippy --workspace --all-targets --features barrier-kit/shell,barrier-kit/tracy -- -D warnings
    "$script_dir/scripts/check-design-tokens.sh"
    "$script_dir/scripts/check-client-only-deps.sh"
    cargo test --workspace --features barrier-kit/shell,barrier-kit/tracy
}

case "${1:-}" in
    app)
        shift
        app "$@"
        ;;
    check)
        check
        ;;
    tracy)
        shift
        tracy "$@"
        ;;
    help|--help|-h|'')
        help
        ;;
    *)
        fail 1 "unknown command: $1"
        ;;
esac
