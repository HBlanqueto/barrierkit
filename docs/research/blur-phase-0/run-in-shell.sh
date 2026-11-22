#!/bin/sh
# usage: run-in-shell.sh <script of this directory> [args]
# Runs the script in the nix dev shell of container `bk`: the kit's shell plus the native
# libraries of Malaysia (shell.nix), with a cargo target directory of its own.
set -eu
here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH='' cd -- "$here/../../.." && pwd)
rel=${here#"$root"/}
podman exec bk sh -c "cd /work && export CARGO_TARGET_DIR=/target/blur-phase-0 CARGO_HOME=/cargo BLURP0=/work/$rel SCRATCH=/work/target/blur-phase-0 && \
    nix develop --impure --expr \"\$(cat /work/$rel/shell.nix)\" -c sh /work/$rel/$*"
