#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail=0

absent() {
  local package=$1
  shift
  if cargo tree "$@" -i "$package" >/dev/null 2>&1; then
    echo "check-client-only-deps: $package must not be in the graph ($*)" >&2
    fail=1
  fi
}

absent winit -p barrier-kit
absent tokio --workspace
absent gtk --workspace
absent glutin --workspace
absent tray-icon --workspace

if cargo tree --workspace -i x11rb >/dev/null 2>&1; then
  echo "check-client-only-deps: WARNING: x11rb reached the graph via freya-clipboard; use wl_data_device instead" >&2
fi
if cargo tree --workspace -i copypasta >/dev/null 2>&1; then
  echo "check-client-only-deps: WARNING: copypasta reached the graph via freya-clipboard; the kit's clipboard is empty for now" >&2
fi

if cargo tree -d -p barrier-kit 2>/dev/null | grep -q '^[a-z]'; then
  echo "check-client-only-deps: duplicated crates in barrier-kit:" >&2
  cargo tree -d -p barrier-kit >&2 || true
fi

exit "$fail"
