# 1. What the shell kit is

**Goal:** know what the shell kit is, which crates to depend on, and how a
shell component differs from an app.
**Scope:** from the crates in the repository to the choice of an entry point.
The first component is in [2](02-first-component.md); each part of the desktop
has its chapter, 3 to 9.
**Reader:** whoever writes a part of Malaysia's desktop: its shell,
compositor, portal or System Settings. It assumes the app guide's chapters
[1](../barrierkit/01-what-is-barrierkit.md) and
[2](../barrierkit/02-first-app.md).
**Input:** nothing. **Output:** the crates to depend on and the entry point to
run.

## Steps

1. Check that the machine has what the shell kit needs:
   [What it needs](#what-it-needs).
2. Depend on `bk-shell` to draw a surface of the desktop, and on `ctshell` to
   read the desktop's settings, glass or tokens ([The crates](#the-crates)).
3. Pick the entry point and the chapters of what you build:
   [What you can build](#what-you-can-build).
4. Draw the first panel: [2. Your first component](02-first-component.md).

## In one sentence

The shell kit puts the app kit on the desktop's own surfaces: the menu bar,
the Dock, the lock screen. A shell component is written like an app's view,
but it runs in a layer surface or a lock surface instead of a window.

## The crates

The shell kit is three crates of the
[`barrierkit`](https://github.com/HBlanqueto/barrierkit) workspace. Apps never
depend on the last two.

| Crate | What it is | Depends on | Who depends on it |
|---|---|---|---|
| `bk`, feature `shell` | The app kit. The hidden `shell` feature adds `bk::extension`: the runner's extension points | — | `bk-shell` only; an app never turns it on |
| `bk-shell` (lib `bk_shell`) | The desktop shell's components on the kit: layer surfaces, the lock screen, the desktop's window list, activation, capture, the status-item host, GTK menu addresses, the Infinity window manager and the shell theme. Linux only | `bk` with `shell`, `ctappearance`, `ctdesign`, `infinity-protocol` with `client` and `shell` | Malaysia's shell |
| `ctshell` | The desktop's data and math, without Freya: its settings files, the wallpaper catalog, the shell glass and Liquid Glass math, and the `ctshell::design` tokens that `build.rs` generates from `crates/ctdesign/tengen/*.json` | `ctappearance`, `ctdesign` | Malaysia's compositor, portal, System Settings and shell |

`ctdesign`, `ctappearance` and `infinity-protocol` are described in
[1, The crates](../barrierkit/01-what-is-barrierkit.md#the-crates).
`scripts/check-client-only-deps.sh` fails if `ctshell` pulls Freya or Skia, so
the compositor can depend on it.

## How it joins the runner

`bk-shell` depends on `bk` with `shell`. Cargo unifies features, so the
shell's own crate lists `bk` with no features. `bk::extension` is hidden from
the docs and is not an API for apps.

`bk`'s `platform/` never imports `bk-shell`. `run_layer` and `run_lock` open
the kit's Wayland runtime (`Runtime::connect`), install one `Extension` in it
and run the kit's event loop. The `Extension` holds the shell's state and five
hooks:

| Hook | When the runner calls it | What `bk-shell` does |
|---|---|---|
| `provide_contexts` | A root is created: the surface or one of its popups | Provides `Toplevels`, `Activation`, the `ScreenLock` state and, under `run_lock`, `SessionUnlock` |
| `new_seat` | A seat appears | Keeps it for the actions of `Toplevels` |
| `new_output` | An output appears | Gives it a lock surface while `run_lock` holds the lock |
| `poll` | Each turn of the loop | Asks for the activation tokens that `Activation::request` queued |
| `kit_event` | A component calls `set_layer_geometry`, `set_card_frames`, `set_plate_frames`, `set_input_region`, `set_blur_region`, `set_minimize_target` or `capture_window` | Applies the request to the surface that sent it |

## A shell component and an app

| | App | Shell component |
|---|---|---|
| Entry point | `Application::new(id).window(..).run()` | `run_layer(app, spec)` or `run_lock(app)` |
| Root | The closure of `.window(..)` | An `AppComponent` (`freya_core::integration::AppComponent`) |
| Surface | An `xdg_toplevel` window | A `zwlr_layer_surface_v1` ([3](03-layer-surfaces.md)) or one `ext_session_lock_surface_v1` per output ([5](05-lock-screen.md)) |
| Place and size | The compositor places the window | The component gives a layer, anchors, an exclusive zone and a size |
| What the kit draws | The window background, the title bar and the traffic lights | Nothing: the surface starts transparent |
| Theme | `Application::theme`, else `Theme::tengen()` | `Theme::tengen()`; Freya's own components take [`use_shell_theme`](07-glass-and-blur.md) |
| Contexts | Windows, menus and app services | `Toplevels`, `Activation`, `use_screen_lock`, `SessionUnlock` |
| Ends | When its last main window closes | When the compositor closes the surface, or after `SessionUnlock::unlock()` |
| Systems | Linux and macOS | Linux only |

## What you can build

| Part of the desktop | What it uses | Chapters |
|---|---|---|
| Menu bar | `run_layer` on `Layer::Top` with `Anchors::TOP` and an exclusive zone; `Toplevels` for the active app and its menus; `use_status_items`; `use_screen_lock`; `MenubarConfig` | [3](03-layer-surfaces.md), [4](04-desktop-windows.md), [5](05-lock-screen.md), [6](06-status-items.md), [8](08-desktop-settings.md) |
| Dock | `run_layer` with `Anchors::BOTTOM`; `set_input_region`, `set_plate_frames`, `set_minimize_target`, `capture_window`; `DockConfig` | [3](03-layer-surfaces.md), [4](04-desktop-windows.md), [8](08-desktop-settings.md) |
| Lock screen | `run_lock` and `SessionUnlock` | [5](05-lock-screen.md) |
| Notification banners | `run_layer` that grows with `set_layer_geometry` and asks for frames with `set_card_frames`; `NotificationsConfig` | [3](03-layer-surfaces.md), [8](08-desktop-settings.md) |
| Launcher | `run_layer` on `Layer::Overlay` with `KeyboardInteractivity::Exclusive`; `Activation` for the app it opens; the `LAUNCHER_*` tokens | [3](03-layer-surfaces.md), [4](04-desktop-windows.md), [9](09-shell-tokens.md) |
| Wallpaper | `run_layer` on `Layer::Background`; `ctshell::wallpaper` | [3](03-layer-surfaces.md), [8](08-desktop-settings.md) |
| Tray host | `use_status_items`, `TrayItem` and `StatusHost` | [6](06-status-items.md) |
| The glass of any of them | `set_blur_region`, `ctshell::shell_glass`, `ctshell::liquid_glass` | [7](07-glass-and-blur.md) |

## What it needs

| Need | Why |
|---|---|
| Linux | `bk::extension` re-exports the Wayland runtime of `crates/bk/src/platform/linux/`, so `bk-shell` does not build on macOS. `ctshell` builds on both ([10](10-run-and-debug.md#what-cannot-run-on-macos)). |
| A Wayland compositor with `zwlr_layer_shell_v1` for `run_layer`, or `ext_session_lock_manager_v1` for `run_lock` | Without it the entry point returns [`RunError::MissingGlobal`](../barrierkit/08-windows.md#errors). Every other global is optional ([10, Globals](10-run-and-debug.md#globals)). |
| What an app needs: `libvulkan`, a Vulkan driver and `nix develop` | The same runner draws the surface ([1, What it needs](../barrierkit/01-what-is-barrierkit.md#what-it-needs)). |

Next: [2. Your first component](02-first-component.md).
