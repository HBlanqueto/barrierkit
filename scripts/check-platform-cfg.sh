#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

violations=$(grep -rn 'target_os' --include='*.rs' crates \
    | grep -Ev '^crates/(bk/src/platform/|bk/tests/|ctappearance/)' || true)

if [[ -n "$violations" ]]; then
    echo "target_os belongs in crates/bk/src/platform/, crates/ctappearance/ and Cargo.toml:" >&2
    echo "$violations" >&2
    exit 1
fi
echo "platform cfg: ok"
