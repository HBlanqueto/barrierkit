#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

blocks=$(mktemp -d)
trap 'rm -rf "$blocks"' EXIT

awk -v dir="$blocks" '
    /^```rust$/ { n++; file = dir "/" n ".rs"; inside = 1; next }
    /^```$/ { inside = 0; next }
    inside { print > file }
' docs/barrierkit/02-first-app.md

status=0
grep -lq 'fn main' "$blocks"/*.rs 2>/dev/null || {
    printf 'check-recipes: no program found in 02-first-app.md\n' >&2
    status=1
}
for recipe in agents/recipes/*.rs; do
    grep -q "recipes/$(basename "$recipe")\"" crates/bk/Cargo.toml || {
        printf 'check-recipes: %s has no [[example]] in crates/bk/Cargo.toml\n' "$recipe" >&2
        status=1
    }
done
for block in "$blocks"/*.rs; do
    grep -q 'fn main' "$block" 2>/dev/null || continue
    found=0
    for recipe in agents/recipes/*.rs; do
        cmp -s "$block" "$recipe" && found=1 && break
    done
    if [ "$found" = 0 ]; then
        printf 'check-recipes: a program in 02-first-app.md is not a file of agents/recipes/\n' >&2
        sed 's/^/  /' "$block" >&2
        status=1
    fi
done
exit "$status"
