#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

broken=$(git ls-files '*.md' | while read -r file; do
    awk '/^[[:space:]]*```/ { fenced = !fenced; next }
        !fenced {
            line = $0
            while (match(line, /\]\([^)[:space:]]+\)/)) {
                print FILENAME "\t" substr(line, RSTART + 2, RLENGTH - 3)
                line = substr(line, RSTART + RLENGTH)
            }
        }' "$file"
done | while IFS="$(printf '\t')" read -r file target; do
    case $target in
        *:* | '#'*) continue ;;
    esac
    [ -e "$(dirname -- "$file")/${target%%#*}" ] || printf '%s: %s\n' "$file" "$target"
done)

if [ -n "$broken" ]; then
    printf 'check-links: broken relative link\n%s\n' "$broken" >&2
    exit 1
fi
