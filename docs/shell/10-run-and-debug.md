# 10. Run and debug

**Goal:** check a change to the shell kit, run a shell component and read its
log.
**Scope:** from `nix develop` to a component that runs nested under a
compositor, with its log. Where each frame's time goes is in
[19. Profiling](../barrierkit/19-profiling.md).
**Reader:** whoever writes a shell component or changes `bk-shell` or
`ctshell`.
**Input:** the repository on Linux with Vulkan, and a Wayland session.
**Output:** a checked change, and a component that runs with its log.

## Steps

1. Enter the dev shell: `nix develop`. `./dev.sh` refuses to run outside it.
2. Check the change: `./dev.sh check`. On Linux it runs clippy with
   `-D warnings` over the whole workspace with `bk/shell,bk/tracy`, so
   `bk-shell`, its examples and `ctshell` compile; then the
   `scripts/check-*.sh`. `scripts/check-recipes.sh` fails if the program of
   [2](02-first-component.md) is not an exact copy of
   `crates/bk-shell/examples/hello_panel.rs`. Its steps are `check()` in
   [`dev.sh`](../../dev.sh).
3. Open a compositor that offers the component's globals, nested in your
   session: [Run nested](#run-nested).
4. Add `bk::init_tracing();` as the first line of the example's `main`, and do
   not commit it: [Logs](#logs).
5. Run the example on that compositor. The examples are the files of
   `crates/bk-shell/examples/`:

   ```sh
   WAYLAND_DISPLAY=wayland-1 RUST_LOG=bk_shell=debug cargo run -p bk-shell --example hello_panel
   ```

6. If the surface does not appear, read the log against [Globals](#globals)
   and [Logs](#logs).

## Run nested

A nested compositor runs as a window of your session, so a component under
test never replaces your own menu bar or locks your own session. A lock screen
runs nested only: under `ext-session-lock-v1`, if the client dies while the
session is locked, the session stays locked
([ext-session-lock-v1](https://wayland.app/protocols/ext-session-lock-v1)).

1. Pick a compositor that offers the globals of [Globals](#globals). Sway and
   labwc offer `zwlr_layer_shell_v1` and `ext_session_lock_manager_v1`; GNOME's
   Mutter offers neither
   ([wlr-layer-shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1),
   [ext-session-lock-v1](https://wayland.app/protocols/ext-session-lock-v1)).
   The dev shell has none of them: use the system's, or
   `nix run nixpkgs#sway`.
2. Start it from your Wayland session. Sway and labwc then open as a window.
3. Find its socket: the new `wayland-N` in `$XDG_RUNTIME_DIR`
   (`ls "$XDG_RUNTIME_DIR"/wayland-*`).
4. Run the component with `WAYLAND_DISPLAY` set to that socket, as in step 5
   of [Steps](#steps).

Under Malaysia, [Try it under Malaysia](../barrierkit/18-platform.md#try-it-under-malaysia)
starts the compositor and reads its socket. Only Malaysia serves the
`infinity_*` globals.

## Logs

The runner and `bk-shell` log through `tracing`, but nothing prints until
`main` calls `bk::init_tracing()`
([19, `init_tracing`](../barrierkit/19-profiling.md#init_tracing)). It prints
to stdout and filters by `RUST_LOG`: `info` when the variable is unset or does
not parse.

A target in `RUST_LOG` matches every target that starts with it, the longest
match wins, and a target it does not match prints nothing:

| `RUST_LOG` | Shows at `debug` |
|---|---|
| `bk_shell=debug` | `bk-shell`, with its failed status-item calls |
| `bk=debug` | The kit's runner and `bk-shell`: `bk` also matches `bk_shell` |
| `bk=info,bk_shell=debug` | `bk-shell` only; the runner stays at `info` |

`WAYLAND_DEBUG=1` prints every request and event between the component and the
compositor.

| Message | Level | Means |
|---|---|---|
| `zwlr_layer_shell_v1 unavailable; Layer surfaces do not work here` | info | `run_layer` returns `MissingGlobal("zwlr_layer_shell_v1")` |
| `zwlr_foreign_toplevel_manager_v1 unavailable; using ext_foreign_toplevel_list_v1` | info | `Toplevels` lists only `id`, `title` and `app_id`, and no action works ([4](04-desktop-windows.md)) |
| `infinity_window_manager_v1 unavailable; continuing without it` | info | The Infinity actions of `Toplevels` do nothing |
| `BarrierKit: ignored a layer geometry with an unanchored zero size` | warn | A `set_layer_geometry` had a 0 without both of its anchors ([3](03-layer-surfaces.md)) |
| `BarrierKit: the compositor ended the session lock` | error | The compositor refused or ended the lock; the process exits with status 1 ([5](05-lock-screen.md)) |
| `BarrierKit: no session bus; no status items` | warn | `use_status_items` hosts nothing ([6](06-status-items.md)) |

The errors that `run_layer` and `run_lock` return are in
[8, Errors](../barrierkit/08-windows.md#errors).

## Globals

`bk-shell` binds these beside the runner's own
([18, Wayland protocols](../barrierkit/18-platform.md#wayland-protocols)).

| Global | Version | Used by | Without it |
|---|---|---|---|
| `zwlr_layer_shell_v1` | 1–4 | `run_layer` | `MissingGlobal("zwlr_layer_shell_v1")` |
| `ext_session_lock_manager_v1` | 1 | `run_lock` | `MissingGlobal("ext_session_lock_manager_v1")` |
| `zwlr_foreign_toplevel_manager_v1` | 3 | `Toplevels` | `Toplevels` falls back to `ext_foreign_toplevel_list_v1` |
| `ext_foreign_toplevel_list_v1` | 1 | `Toplevels` when the wlr manager is missing | No windows |
| `infinity_window_manager_v1` | 1–8 | The Infinity actions of `Toplevels`, its `focus_bar` and `desktop`, `restorable` and the menu addresses | They do nothing or stay empty |
| `infinity_session_v1` | 1 | `use_screen_lock` | It stays `ScreenLock::Unlocked` |
| `ext_background_effect_manager_v1` | 1 | `set_blur_region` | No blur |
| `ext_image_copy_capture_manager_v1`, `ext_foreign_toplevel_image_capture_source_manager_v1`, `ext_foreign_toplevel_list_v1` and `wl_shm` | 1 | `capture_window` | `capture_window` calls back with `None` |
| `xdg_activation_v1`, the runner's | 1 | `Activation::request` | The callback gets `None` |
| `infinity_window_style_v1`, the runner's | 4, 8 | `set_card_frames` from version 4, `set_plate_frames` from version 8 | No frames and no plates |

## What cannot run on macOS

1. `bk-shell` and `bk`'s `shell` feature do not compile on macOS:
   `bk::extension` re-exports the Wayland runtime of
   `crates/bk/src/platform/linux/`. On macOS `./dev.sh check` passes
   `--exclude bk-shell` and only the `bk/tracy` feature.
2. `ctshell` compiles on macOS: the macOS `./dev.sh check` covers it.
3. No shell component runs on macOS, and no example of
   `crates/bk-shell/examples/` builds there.
4. So a change to `bk-shell`, to `crates/bk/src/platform/linux/` or to code
   under `cfg(feature = "shell")` made on a Mac is compiled only by a Linux
   `./dev.sh check`. [Linux verification](../barrierkit/linux-verification.md)
   is the procedure for it.

Back to the [index](README.md).
