#!/bin/sh
# usage: rerun.sh [malaysia checkout] [base rev]
# Rebuilds the scratch tree of Malaysia from compositor-within.patch and runs the
# experiment on software EGL in container `bk` (the repository is mounted at /work).
set -eu
here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH='' cd -- "$here/../../.." && pwd)
scratch=$root/target/blur-phase-0
malaysia=${1:-$HOME/src/malaysia}
base=${2:-1a45e1159039478d5c9e5dedc17d1844f91a497f}

podman exec bk true 2>/dev/null || {
    podman machine start barrierkit >/dev/null 2>&1 || true
    podman start bk >/dev/null
}
rm -rf "$scratch/malaysia" "$scratch/frames"
mkdir -p "$scratch/malaysia" "$scratch/frames"
git -C "$malaysia" archive "$base" | tar -x -C "$scratch/malaysia"
patch -s -p1 -d "$scratch/malaysia" < "$here/compositor-within.patch"

sh "$here/run-in-shell.sh" run-within.sh | tee "$scratch/results.log" | grep -E '^BLURP0|^test |test result'
sh "$here/run-in-shell.sh" run-suite.sh
