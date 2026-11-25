#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)

require_devshell() {
    [ "$(uname -s)" = Darwin ] || [ "${BARRIERKIT_DEV_SHELL:-}" = "1" ] ||
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
  check                   run fmt, clippy and kit-specific checks
  macos-smoke [recipe...] open each recipe as a macOS app and check its window (macOS only)
  help                    show this message

All commands require 'nix develop', except on macOS: there they use the
user's Rust toolchain with rustfmt and clippy (docs/barrierkit/18-platform.md).
EOF
}

check() {
    require_devshell
    require cargo
    require cargo-fmt
    require cargo-clippy
    features=bk/shell,bk/tracy
    allow=
    if [ "$(uname -s)" = Darwin ]; then
        # Window and popup commands carry fields and the menu handle keeps the path and the
        # action queue that only the Linux backend reads.
        allow='-A dead_code'
    fi
    cargo fmt --all -- --check
    if command -v shellcheck >/dev/null 2>&1; then
        shellcheck -x "$script_dir/dev.sh" "$script_dir"/scripts/*.sh
    fi
    # shellcheck disable=SC2086
    {
        cargo clippy --locked -p bk --lib -- -D warnings $allow
        cargo clippy --locked -p bk --lib --no-default-features -- -D warnings $allow
        cargo clippy --locked --workspace --lib --examples --tests --features "$features" -- -D warnings $allow
    }
    cargo clippy --locked -p ctdesign -p ctappearance -p limitless --lib -- -D warnings
    cargo test --locked --workspace --lib --features "$features"
    "$script_dir/scripts/check-design-tokens.sh"
    "$script_dir/scripts/check-client-only-deps.sh"
    "$script_dir/scripts/check-platform-cfg.sh"
    "$script_dir/scripts/check-links.sh"
    "$script_dir/scripts/check-recipes.sh"
    "$script_dir/scripts/check-render-loop.sh"
    "$script_dir/scripts/tengen-css.sh" | diff -u "$script_dir/docs/barrierkit/tengen.css" -
}

case "${1:-}" in
    check)
        check
        ;;
    macos-smoke)
        shift
        "$script_dir/scripts/macos-smoke.sh" "$@"
        ;;
    help|--help|-h|'')
        help
        ;;
    *)
        fail 1 "unknown command: $1"
        ;;
esac
