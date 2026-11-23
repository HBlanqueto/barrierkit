#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# shortcut: these files still mix both systems; delete a line when its cfg moves into crates/bk/src/platform/.
pending='crates/bk/src/(scroll|portal|main_menu|notifications|secrets)\.rs|crates/bk/src/app/(application|defaults)\.rs'

violations=$(grep -rn 'target_os' --include='*.rs' crates \
    | grep -Ev '^crates/(bk/src/platform/|bk/tests/|ctappearance/)' \
    | grep -Ev "^($pending):" || true)

if [[ -n "$violations" ]]; then
    echo "target_os belongs in crates/bk/src/platform/, crates/ctappearance/ and Cargo.toml:" >&2
    echo "$violations" >&2
    exit 1
fi
echo "platform cfg: ok"
