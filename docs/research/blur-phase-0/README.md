# Blur phase 0: the experiment, ready to rerun

**Goal:** keep the phase 0 experiment of
[`blur-backdrop-design.md`](../blur-backdrop-design.md#72-phase-0-results) in
the repository, so anyone can rerun it and see the numbers it printed.
**Scope:** the scratch compositor code, the scripts that run it and the output
of its last run. It starts at a Malaysia checkout and ends at the test output.
It is not the compositor implementation: phase 2 writes that.
**Reader:** whoever reruns phase 0 or runs the GPU measurement of
[section 10](../blur-backdrop-design.md#10-gpu-runbook-the-frame-time-of-the-band-pass).
**Input:** a Malaysia checkout with the base revision `1a45e115`, and either
the container of step 1 (software) or a Linux machine with a GPU (section 10).
**Output:** the lines `BLURP0 ...` and the test results.

## Files

| File | What it is |
|---|---|
| `compositor-within.patch` | `diff -ruN`, `-p1`, against Malaysia `1a45e1159039478d5c9e5dedc17d1844f91a497f` (Smithay `65aa86f`). It adds `blur/within.rs` (the split and the blur, about 130 lines) and `blur/within/tests.rs` (a Wayland client over a socket pair, a lockstep harness and 10 tests), and changes `blur.rs` so `render_blurred` takes any slice of render elements |
| `rerun.sh` | Rebuilds the scratch tree from the patch and runs everything in the container |
| `run-in-shell.sh` | Runs a script of this directory in the container's nix shell: the kit's shell plus the native libraries of Malaysia (`shell.nix`) |
| `run-within.sh`, `run-suite.sh` | The 11 experiment tests, one thread; the rest of Malaysia's tests three times, four threads, without the experiment |
| `swegl.sh` | Sets `__EGL_VENDOR_LIBRARY_DIRS` so Mesa shows its software device to Smithay. Only the software runs source it |
| `egl-probe.c`, `egl-probe.sh` | Lists the EGL devices in Smithay's order and prints each one's `GL_RENDERER` |
| `results.log` | The output of `run-within.sh` on 2026-11-21 (llvmpipe) |

The patch differs from the one that produced the numbers of the first phase 0
run in two test-only lines: the `TIMING` line prints the renderer's
`GL_RENDERER` and the share of a 60 Hz and a 120 Hz frame, and says
`skipped` when there is no EGL device. The first run printed a fixed
"software rendering" label. The measured code is the same.

## Steps

1. Start the container. It needs a podman machine `barrierkit` with a container
   `bk` (image `nixos/nix`) that mounts this repository at `/work`. It is a
   local setup, not part of the repository. `rerun.sh` starts both if they are
   stopped.
2. Run it. Cold, it takes about 1.5 minutes; warm, about 35 seconds.

   ```sh
   sh docs/research/blur-phase-0/rerun.sh [/path/to/malaysia] [base-rev]
   ```

   The checkout defaults to `$HOME/src/malaysia`. The script only reads it
   (`git archive`). It writes the scratch tree, the dumped frames and the full
   log to `target/blur-phase-0/`, which git ignores.
3. Read the output. The last lines are the runs of `run-suite.sh`: Malaysia's own
   288 library tests and 15 binary tests, which must pass three times. Above
   them are the 10 tests of the experiment (the filter `within` also matches one
   unrelated input test, so the run reports 11). Each prints one line
   `BLURP0 <layout>-<band>-<1x|2x> ...` with these fields:

   | Field | Pass when |
   |---|---|
   | `outside_mismatch` | 0: no pixel outside the band differs from the sharp content |
   | `marks_sharp` | equals `frames`: the client's own pixels over the band stay sharp |
   | `equals_fresh` | equals `frames`: the band equals a from-scratch render of that frame |
   | `incremental_age1_equals_full_repaint` | equals `frames`: damage tracking repaints the band correctly |
   | `mid_equals_previous` | `frames` minus 1: between the sub-surface commit and the parent commit nothing changes |
   | `region_after_parent_commit` | equals `frames`: the new blur region applies with the parent's commit |
   | `mid_torn`, `mid_region_new` | 0 with a synchronized sub-surface; above 0 in the desynchronized control |

   The two tests `the_band_blur_pass_costs_this_much_in_software_at_1x` and
   `_at_2x` print the extra `BLURP0 TIMING` line. Its numbers are llvmpipe's,
   not a GPU's.
4. To measure on a GPU, follow
   [section 10](../blur-backdrop-design.md#10-gpu-runbook-the-frame-time-of-the-band-pass).
   That run does not use `swegl.sh`.
