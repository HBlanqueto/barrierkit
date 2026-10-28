#!/usr/bin/env sh
# BarrierKit developer helper.
#
# All build commands must run inside `nix develop` so the Skia prebuild,
# linker flags and library paths are consistent (see README).
set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)

# require_devshell - refuse to run outside the Nix dev shell.
require_devshell() {
    [ "${BARRIERKIT_DEV_SHELL:-}" = "1" ] ||
        fail 2 "not inside the dev shell; run 'nix develop' first"
}

# require - fail if a command is missing.
require() {
    command -v "$1" >/dev/null 2>&1 || fail 2 "'$1' not found"
}

# fail - print a message and exit with a code.
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
  examples                list available examples in crates/barrier-kit/examples
  example <name> [args]   build and run one example against the current WAYLAND_DISPLAY
  check                   run fmt, clippy, tests and kit-specific checks
  help                    show this message

All commands require 'nix develop'.
EOF
}

examples() {
    require_devshell
    printf 'Available examples:\n'
    for _f in "$script_dir"/crates/barrier-kit/examples/*.rs; do
        [ -f "$_f" ] || continue
        basename "$_f" .rs
    done
}

example() {
    require_devshell
    require cargo
    if [ "$#" -eq 0 ]; then
        fail 1 "missing example name; run './dev.sh examples' to list them"
    fi
    _name=$1
    shift
    if [ ! -f "$script_dir/crates/barrier-kit/examples/$_name.rs" ]; then
        fail 1 "no such example: $_name"
    fi
    RUST_LOG=${RUST_LOG:-info} \
        cargo run -p barrier-kit --example "$_name" -- "$@"
}

check() {
    require_devshell
    require cargo
    require cargo-fmt
    require cargo-clippy
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    "$script_dir/scripts/check-design-tokens.sh"
    "$script_dir/scripts/check-client-only-deps.sh"
    cargo test --workspace
}

case "${1:-}" in
    examples)
        examples
        ;;
    example)
        shift
        example "$@"
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
