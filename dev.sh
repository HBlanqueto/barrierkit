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
  check                   run fmt, clippy, tests and kit-specific checks
  help                    show this message

All commands require 'nix develop'.
EOF
}

check() {
    require_devshell
    require cargo
    require cargo-fmt
    require cargo-clippy
    cargo fmt --all -- --check
    cargo clippy -p bk --lib -- -D warnings
    cargo clippy -p bk --lib --no-default-features -- -D warnings
    cargo clippy --workspace --all-targets --features bk/charts,bk/print,bk/shell,bk/tracy,bk/video -- -D warnings
    "$script_dir/scripts/check-design-tokens.sh"
    "$script_dir/scripts/check-client-only-deps.sh"
    "$script_dir/scripts/check-links.sh"
    "$script_dir/scripts/check-recipes.sh"
    "$script_dir/scripts/tengen-css.sh" | diff -u "$script_dir/agents/tengen.css" -
    cargo test --workspace --lib --tests --features bk/charts,bk/print,bk/shell,bk/tracy,bk/video
}

case "${1:-}" in
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
