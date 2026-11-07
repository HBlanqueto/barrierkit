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
for block in "$blocks"/*.rs; do
    grep -q 'fn main' "$block" || continue
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
