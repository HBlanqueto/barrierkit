#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

status=0
report() {
    printf 'design-tokens: %s\n' "$1" >&2
    shift
    printf '%s\n' "$@" >&2
    status=1
}

# Demo content (Marea's catalogue art, Ondina's album covers) is data, not design.
production() {
    for file in $(find "$@" -name '*.rs' ! -path 'crates/vow-design/*' \
        ! -path 'crates/marea/src/catalog.rs' ! -path 'crates/ondina/src/albums.rs' | sort); do
        awk '/#\[cfg\(test\)\]/{exit} {print FILENAME ":" FNR ":" $0}' "$file"
    done
}

if matches=$(production crates/*/src |
    grep -E '0x[0-9a-fA-F]{6,8}|Rgba::(rgb8|rgba8)\([[:space:]]*(0x)?[0-9]'); then
    report "colour literals must live in vow-design" "$matches"
fi

if matches=$(production crates/*/src |
    grep -E '\[u8; ?4\]'); then
    report "convert colours with Rgba::to_rgba8, not a local [u8; 4]" "$matches"
fi

if matches=$(production \
    crates/barrier-kit/src/password.rs \
    crates/barrier-kit/src/controls.rs \
    crates/barrier-kit/src/theme.rs |
    grep -E ':[0-9]+:[[:space:]]*const [A-Z0-9_]+: (i32|u32|f32) = [0-9]'); then
    report "UI measurements must live in vow-design" "$matches"
fi

exit "$status"
