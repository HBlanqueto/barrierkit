# 4. Desktop windows

**Goal:** list the windows of every app and act on them from a shell surface:
focus, close, minimize, tile, capture and force quit them, and read the shown
space and the menus of the active window.
**Scope:** `Toplevels`, `ToplevelInfo` and `FocusBar`, the window actions,
`set_minimize_target` and `MinimizeTarget`, `capture_window`,
`CaptureWindow` and `WindowImage`, the Infinity window manager
(`InfinityLayout`, `InfinityPairLayout`, `InfinityWindowManagerV1`), the menu
addresses `MainMenuAddress` and `GtkMenusAddress`, and `Activation`. An app's
own window is in [8. Windows](../barrierkit/08-windows.md); the surface the
component runs in is in [3. Layer surfaces](03-layer-surfaces.md).
**Reader:** whoever writes Malaysia's Dock, menu bar, app switcher or window
overview.
**Input:** a component that runs in a layer surface ([3](03-layer-surfaces.md))
or a lock surface ([5](05-lock-screen.md)).
**Output:** a component that shows the desktop's windows and acts on them.

## Steps

1. Read the windows with `Toplevels::get().list.read()`
   ([`Toplevels`](#toplevels)).
2. Act on a window by its `id`: `activate`, `close`, `minimize`
   ([Window actions](#window-actions)), or tile it through Infinity
   ([Infinity window manager](#infinity-window-manager)).
3. Tell the compositor where each window minimizes to with
   [`set_minimize_target`](#minimize-target).
4. Draw a thumbnail with [`capture_window`](#thumbnails).
5. Follow the shown space with `focus_bar` and `desktop`
   ([Spaces](#spaces-the-focus-bar-and-the-shown-desktop)).
6. In the menu bar, read the menus of the active window from its
   [menu address](#menu-addresses).
7. Before you launch an app, ask for an [activation token](#activation).

This program is `crates/bk-shell/examples/dock_windows.rs`. A bar on the
bottom edge shows a button per window, marks the active one, and focuses a
window on a press:

```rust
use bk::{
    prelude::*,
    theme::{SurfaceKind, resolved, surface_background},
};
use bk_shell::{Anchors, KeyboardInteractivity, Layer, LayerSpec, Toplevels, run_layer};
use freya_core::integration::AppComponent;

const HEIGHT: u32 = 48;

fn dock() -> impl IntoElement {
    let toplevels = Toplevels::get();
    let windows = toplevels.list.read().clone();
    HStack::new()
        .expanded()
        .background(color(surface_background(
            &resolved(),
            SurfaceKind::LayerShell,
        )))
        .children(windows.into_iter().map(|window| {
            let toplevels = toplevels.clone();
            Button::new(window.title.clone())
                .toggled(window.active)
                .key(window.id)
                .on_press(move |_| {
                    if window.minimized {
                        toplevels.unset_minimized(window.id);
                    }
                    toplevels.activate(window.id);
                })
        }))
}

fn main() -> Result<(), RunError> {
    run_layer(
        AppComponent::from(dock),
        LayerSpec {
            namespace: "dock-windows".to_string(),
            layer: Layer::Top,
            anchors: Anchors::BOTTOM,
            exclusive_zone: HEIGHT as i32,
            keyboard: KeyboardInteractivity::None,
            size: Some((0, HEIGHT)),
        },
    )
}
```

## Globals

| Global | Version | Gives | Without it |
|---|---|---|---|
| `zwlr_foreign_toplevel_manager_v1` | 3 | The list with each window's state, and every action | The kit uses `ext_foreign_toplevel_list_v1` |
| `ext_foreign_toplevel_list_v1` | 1 | Bound only without the wlr manager: `id`, `title` and `app_id` | No windows |
| `infinity_window_manager_v1` | 1–8 | Tiling, restore, full screen, force quit, shortcuts, the shown space and the menu addresses | Those actions do nothing and those fields keep their defaults |
| `ext_image_copy_capture_manager_v1`, `ext_foreign_toplevel_image_capture_source_manager_v1`, a second `ext_foreign_toplevel_list_v1` and `wl_shm` | 1 | `capture_window` | `capture_window` answers `None` |
| `xdg_activation_v1`, the runner's | 1 | `Activation` | `Activation::request` answers `None` |

The kit logs a missing wlr manager and a missing Infinity manager at `info`
level, and continues.

## `Toplevels`

`Toplevels::get()` returns the windows of every app. It is a root context of
each surface that `run_layer` and `run_lock` create, and of their popups;
elsewhere it panics.

| Field | Type | Holds |
|---|---|---|
| `list` | `State<Vec<ToplevelInfo>>` | The windows, in the order they appeared |
| `focus_bar` | `State<FocusBar>` | The bar over the shown space ([Spaces](#spaces-the-focus-bar-and-the-shown-desktop)) |
| `desktop` | `State<Option<u32>>` | The number of the shown desktop ([Spaces](#spaces-the-focus-bar-and-the-shown-desktop)) |

| `ToplevelInfo` field | Type | Meaning |
|---|---|---|
| `id` | `u64` | The window's id for its life; every action takes it |
| `title`, `app_id` | `String` | As the app sets them |
| `active`, `maximized`, `minimized` | `bool` | The window's state |
| `restorable` | `bool` | `restore` has a saved rectangle |
| `main_menu` | `Option<MainMenuAddress>` | The window's dbusmenu address ([Menu addresses](#menu-addresses)) |
| `gtk_menus` | `Option<GtkMenusAddress>` | The window's GTK menu address ([Menu addresses](#menu-addresses)) |

The runner keeps the list with these rules:

1. A window the compositor announces goes at the end with a new `id`. The ids
   count from 1 and the process never reuses one.
2. A change of title, app id or state changes that entry in place.
3. A window the compositor reports closed leaves the list.
4. Each change replaces the whole list, so a component that reads it renders
   again.

With `ext_foreign_toplevel_list_v1` alone, every entry has only `id`, `title`
and `app_id`, and no action works. The kit does not keep the full-screen
state.

## Window actions

These go through the window's `zwlr_foreign_toplevel_handle_v1`. An `id` that
names no window, or a list from `ext_foreign_toplevel_list_v1`, does nothing.

| Method | Request | Does |
|---|---|---|
| `activate(id)` | `activate`, with the seat | Asks to focus the window; needs a seat |
| `close(id)` | `close` | Asks the window to close |
| `minimize(id)` | `set_minimized` | Minimizes it |
| `unset_minimized(id)` | `unset_minimized` | Restores it from minimized |
| `set_maximized(id, bool)` | `set_maximized`, `unset_maximized` | Maximizes or unmaximizes it |

The compositor and the app decide. `activate` comes with no guarantee, and
`close` only asks: an app with unsaved changes keeps its window
([8. Windows](../barrierkit/08-windows.md#documents-and-file-dialogs)). The
list changes when the compositor reports the change, not when the method
returns. The protocol does not say that `activate` restores a minimized window,
so the example calls `unset_minimized` first.

## Minimize target

`set_minimize_target(MinimizeTarget { window, rect })` tells the compositor
where `window` minimizes to (`zwlr_foreign_toplevel_handle_v1.set_rectangle`):

1. `rect` is `(x, y, width, height)` in the calling surface's coordinates;
   the kit sends that surface with it.
2. A later rectangle replaces the earlier one; a width and height of 0 remove
   it.
3. The compositor takes it as a hint, for example for the minimize effect.

Malaysia's Dock sends the slot of each window's tile.

## Thumbnails

`capture_window(window, done)` copies the pixels of `window` once and calls
`done` with `Some(WindowImage)`, or with `None` when it cannot:

1. It finds the window's `ext_foreign_toplevel_handle_v1`. With the wlr list
   it matches the window in the kit's second `ext_foreign_toplevel_list_v1` by
   app id and title; two windows with the same app id and title give `None`.
2. It creates a capture source and a session (`ext-image-copy-capture-v1`).
3. It waits for the buffer size and the formats. It takes `ARGB8888` or
   `XRGB8888` in shared memory; with neither, `None`.
4. It copies one frame into a `wl_shm` buffer.
5. It calls `done` and redraws every surface. A frame that fails, or a session
   that stops, gives `None`.

| `WindowImage` field | Type | Holds |
|---|---|---|
| `width`, `height` | `u32` | The size in buffer pixels |
| `rgba` | `Vec<u8>` | Four bytes per pixel, R, G, B, A, row after row; an `XRGB8888` frame has A at 255 |

Draw it with `Image::rgba(image.width, image.height, image.rgba)`
([Images](../barrierkit/04-appearance-and-tokens.md#images-and-template-rendering)).
There is no stream: call it again for a newer image. `CaptureWindow` is the
request that `capture_window` sends to the runner; a shell does not build it.
Malaysia's Dock draws the tile of a minimized window this way.

## Infinity window manager

`infinity_window_manager_v1` is Malaysia's private protocol
(`crates/infinity-protocol/protocols/infinity-window-manager-v1.xml`,
[ADR 0039](../decisions.md#0039--infinity-v1)). It names windows by their wlr handle, so the
methods that take an `id` need the wlr list. The compositor ignores a handle
that no longer names a window. `InfinityWindowManagerV1` is its generated
proxy; `Toplevels` calls it, a shell does not.

| Method | Version | Does |
|---|---|---|
| `arrange(id, InfinityLayout)` | 1 | Places the window. The first arrange saves its rectangle; later ones keep it |
| `arrange_pair(InfinityPairLayout)` | 1 | Tiles the active window and the previous visible one in focus order |
| `arrange_quarters()` | 1 | Tiles the four most recent visible windows in quarters |
| `restore(id)` | 1 | Applies the saved rectangle and forgets it |
| `set_fullscreen(id, bool)` | 1 | Covers the whole output, or leaves and restores the previous rectangle |
| `force_quit(id)` | 4 | The compositor kills the window's process with `SIGKILL`; every window of that process goes |
| `send_shortcut(id, keys)` | 6 | The compositor focuses the window and types the chord into it, past its own shortcuts |

Below the version a method needs, it does nothing. `force_quit` is for a hung
app that ignores `close`: ask the user first. `send_shortcut` serves the
shortcut menus of apps that publish no menu. `keys` is the modifiers and one
xkbcommon keysym name joined with `+`, such as `"Ctrl+Shift+S"`; the compositor
ignores it when a key is unknown, the session is locked or the window cannot
take the keyboard.

`ToplevelInfo::restorable` follows the `restorable` event: it is `true` while
`restore` has a rectangle, so the shell can enable a Restore item.

| `InfinityLayout` | Place |
|---|---|
| `Fill` | The workspace, within its margin |
| `Center` | The center, at the current size |
| `Left`, `Right`, `Top`, `Bottom` | A half |
| `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight` | A quarter |

| `InfinityPairLayout` | Active window | Previous window |
|---|---|---|
| `SideBySide` | Left | Right |
| `SideBySideReversed` | Right | Left |
| `Stacked` | Top | Bottom |
| `StackedReversed` | Bottom | Top |

### Spaces: the focus bar and the shown desktop

Malaysia shows either a desktop or a focus space (Malaysia ADR 0051). The
compositor sends both values when the manager binds and whenever they change.

| `Toplevels` field | Event | Version | Value |
|---|---|---|---|
| `focus_bar` | `space` | 5 | `FocusBar::None` on a desktop; `Hidden` over a focus space with the bar up; `Revealed` over a focus space with the bar down, because the pointer is at the top edge |
| `desktop` | `desktop` | 8 | The number of the shown desktop, from 1; `None` over a focus space |

`FocusBar::over_focus()` is `true` for `Hidden` and `Revealed`. Over a focus
space the bar is drawn black with white text. The bar reads the wallpaper of
the shown desktop with `desktop`
([8. Desktop settings](08-desktop-settings.md)). Below version 5 `focus_bar`
stays `None`; below version 8 `desktop` stays `None`.

## Menu addresses

The menu bar shows the menus of the active window. The kit gives the D-Bus
address of each window's menus; the shell reads the menus there. The
compositor sends each address when the window appears, when the manager binds
and when the address changes, never when the focus changes: the shell takes the
address of the entry whose `active` is `true`.

| Field | Event | Version | Holds |
|---|---|---|---|
| `main_menu` | `app_menu` | 2 | `MainMenuAddress { service, path }`: the window's `com.canonical.dbusmenu` menu, which the app announces with `org_kde_kwin_appmenu`. A kit app announces its own ([8](../barrierkit/08-windows.md#menu-address)). `None` when the service or the path is empty |
| `gtk_menus` | `gtk_menus` | 3 | `GtkMenusAddress`: the menus a GTK 3 or GTK 4 app exports |

| `GtkMenusAddress` field | Holds |
|---|---|
| `unique_bus_name` | The app's unique name on the session bus |
| `application_id` | The GTK application id |
| `menubar_path`, `app_menu_path` | Object paths of the menu bar and the app menu, each an `org.gtk.Menus` object |
| `application_object_path`, `window_object_path` | Object paths of the `org.gtk.Actions` groups that the menu items name (`app.` and `win.`) |

`gtk_menus` is `Some` only when `unique_bus_name` and at least one of the two
menu paths are set. Without Infinity, or below the version, both fields stay
`None`.

## Activation

`Activation::get()` returns the activation handle, a root context like
`Toplevels`. `Activation::request(callback)` asks for an `xdg_activation_v1`
token:

1. On the next turn of the event loop, the runner takes the serial of the last
   input event, its seat and the surface that got it, or the first surface.
2. It asks for a token with them.
3. The compositor answers, and the callback gets `Some(token)`.

The callback gets `None` at once when there is no activation global, no seat,
no surface, or no input yet. The shell hands the token to the app it launches,
for example in its `XDG_ACTIVATION_TOKEN` environment variable, so that app may
take the focus. A running kit window takes a token with
`WindowControls::present` ([8](../barrierkit/08-windows.md#window-controls)).

Next: [5. Lock screen](05-lock-screen.md).
