#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

command -v swiftc >/dev/null 2>&1 || {
    printf 'swift-to-bk check: swiftc not found, skipped\n' >&2
    exit 0
}

python3 -I scripts/swift-to-bk/tests/run.py

if [ "${1:-}" = "--build" ]; then
    for recipe in agents/recipes/swift_*.rs; do
        cargo build -p bk --example "$(basename "$recipe" .rs)"
    done
fi
