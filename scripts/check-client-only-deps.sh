#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

host=$(rustc -vV | sed -n 's/^host: //p')
metadata=$(cargo metadata --locked --offline --all-features --format-version 1 --filter-platform "$host")

violations=$(jq -r '
  . as $m
  | ($m.resolve.nodes | map({(.id): [.deps[].pkg]}) | add) as $deps
  | ($m.packages | map({(.id): .name}) | add) as $name
  | def closure:
      . as $ids
      | ([$ids[], ($ids[] | $deps[.][])] | unique) as $next
      | if $next == $ids then $ids else $next | closure end;
    def reach($crate):
      [$m.packages[] | select(.name == $crate and .source == null) | .id]
      | if . == [] then error("\($crate) is not a workspace member") else . end
      | closure | map($name[.]);
  [
    (reach("bk")[] | select(. == "winit") | "bk pulls \(.)"),
    ($name[] | select(test("^(tokio|gtk|glutin|tray-icon|freya|freya-winit|laye-rs|otto-.+)$"))
      | "the workspace pulls \(.)"),
    (("ctdesign", "ctappearance", "infinity-protocol") as $crate
      | reach($crate)[] | select(startswith("freya") or contains("skia"))
      | "\($crate) pulls \(.)")
  ] | unique[]
' <<<"$metadata")

if [ -n "$violations" ]; then
  sed 's/^/check-client-only-deps: /' <<<"$violations" >&2
  exit 1
fi
