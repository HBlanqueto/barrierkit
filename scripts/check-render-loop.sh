#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

version=$(sed -n 's/^freya-core = "=\(.*\)"$/\1/p' Cargo.toml)
expected_version=0.5.3
expected_sha=1abc4ef6808fceb7a084cf781e0a4c8f9187b68f3f60968f706cc75fe12e4422

if [ "$version" != "$expected_version" ]; then
    printf 'check-render-loop: freya-core is %s; compare crates/bk/src/group.rs with its render_pipeline.rs, then update this script\n' "$version" >&2
    exit 1
fi

file=$(find "${CARGO_HOME:-$HOME/.cargo}/registry/src" -path "*/freya-core-$version/src/render_pipeline.rs" 2>/dev/null | head -n 1)
if [ -z "$file" ]; then
    printf 'check-render-loop: SKIP: freya-core %s is not in the cargo registry (run cargo fetch)\n' "$version" >&2
    [ -n "${CI:-}" ] && exit 1
    exit 0
fi

sha=$(sha256sum "$file" 2>/dev/null || shasum -a 256 "$file")
sha=${sha%% *}
if [ "$sha" != "$expected_sha" ]; then
    printf 'check-render-loop: %s changed (%s); compare it with crates/bk/src/group.rs, then update this script\n' "$file" "$sha" >&2
    exit 1
fi
