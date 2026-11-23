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
  check                   run fmt, clippy, tests and kit-specific checks
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
    if [ "$(uname -s)" = Darwin ]; then
        # The Wayland code is unused on macOS and the shell feature does not build there.
        features=bk/tracy
        allow='-A dead_code'
    else
        features=bk/shell,bk/tracy
        allow=
    fi
    cargo fmt --all -- --check
    # shellcheck disable=SC2086
    {
        cargo clippy -p bk --lib -- -D warnings $allow
        cargo clippy -p bk --lib --no-default-features -- -D warnings $allow
        cargo clippy --workspace --all-targets --features "$features" -- -D warnings $allow
    }
    "$script_dir/scripts/check-design-tokens.sh"
    "$script_dir/scripts/check-client-only-deps.sh"
    "$script_dir/scripts/check-links.sh"
    "$script_dir/scripts/check-recipes.sh"
    "$script_dir/scripts/tengen-css.sh" | diff -u "$script_dir/agents/tengen.css" -
    cargo test --workspace --lib --tests --features "$features"
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
