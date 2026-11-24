# bk-shell

**Goal:** open a shell surface (menu bar, Dock, background or lock screen)
with `bk-shell` and read the desktop from it.
**Scope:** from the shell's `Cargo.toml` to a layer or lock surface drawn by
the kit's runner, with the windows, the activation tokens and the status
items of other apps. What the runner does under every surface is in
[18. Platform](../barrierkit/18-platform.md); the window API is in
[8. Windows](../barrierkit/08-windows.md).
**Reader:** whoever writes a component of Malaysia's shell.
**Input:** a root component written as for a window
([2. First app](../barrierkit/02-first-app.md)).
**Output:** a layer surface or a lock screen drawn by BarrierKit, and the
contexts that tell it about the desktop.

`bk-shell` builds on Linux only.

## Steps

1. Depend on `bk-shell` in the shell's `Cargo.toml`, beside `bk` with no
   features:

   ```toml
   bk = { git = "https://github.com/HBlanqueto/barrierkit", rev = "…" }
   bk-shell = { git = "https://github.com/HBlanqueto/barrierkit", rev = "…" }
   ```

2. Write the root component, as for a window. Wrap it in an `AppComponent`
   (`freya_core::integration::AppComponent`):

   ```rust
   let app = AppComponent::from(menu_bar);
   ```

3. Describe the surface with a [`LayerSpec`](#run_layer-and-layerspec):

   ```rust
   let spec = LayerSpec {
       namespace: "malaysia-menubar".to_string(),
       layer: Layer::Top,
       anchors: Anchors::TOP,
       exclusive_zone: MENUBAR_HEIGHT,
       keyboard: KeyboardInteractivity::None,
       size: Some((0, MENUBAR_HEIGHT as u32)),
   };
   ```

4. Run it. `run_layer` returns when the compositor closes the surface:

   ```rust
   run_layer(app, spec)?;
   ```

5. Inside the component, read the desktop through the shell contexts:
   [`Toplevels::get()`](#toplevels), [`Activation::get()`](#activation) and
   [`use_status_items()`](#status-items).

## How it joins the runner

`bk-shell` depends on `bk` with its `shell` feature. That feature adds only
`bk::extension`, hidden from the docs: the runner's extension points for
`bk-shell`. An app never enables it.

`bk`'s `platform/` never imports `bk-shell`. `bk-shell` hooks into the runner
through one `Extension` with five hooks: provide the root contexts, a new
seat, a new output, a poll on each loop turn, and a kit event, which carries
the requests of `set_layer_geometry`, `set_card_frames`, `set_plate_frames`,
`set_input_region`, `set_blur_region`, `set_minimize_target` and
`capture_window` to the surface that sent them
([ADR 0047](../decisions.md)).

### Globals

`bk-shell` binds these, beside the ones the runner binds
([Wayland protocols](../barrierkit/18-platform.md#wayland-protocols)). When an
optional one is missing it logs it at `info` level and continues.

| Global | Version | Used by | Without it |
|---|---|---|---|
| `zwlr_layer_shell_v1` | 1–4 | `run_layer` | `MissingGlobal("zwlr_layer_shell_v1")` |
| `ext_session_lock_manager_v1` | 1 | `run_lock` | `MissingGlobal("ext_session_lock_manager_v1")` |
| `zwlr_foreign_toplevel_manager_v1` | 3 | `Toplevels`, first choice | — |
| `ext_foreign_toplevel_list_v1` | 1 | `Toplevels`, only when the wlr manager is missing | No toplevels |
| `infinity_window_manager_v1` | 1–8 | Tiling, force quit, shortcuts, spaces, the shown desktop and menu addresses in `Toplevels` | The actions marked Infinity do nothing |
| `infinity_session_v1` | 1 | `locking` and `unlocked`, read by [`use_screen_lock`](#screen-lock-and-blur-region) | The menu bar never learns that the session is locking |
| `ext_background_effect_manager_v1` | 1 | [`set_blur_region`](#screen-lock-and-blur-region) | No blur behind the surface |
| `ext_image_copy_capture_manager_v1` + `ext_foreign_toplevel_image_capture_source_manager_v1` + `wl_shm` | 1 | [`capture_window`](#toplevels) | `capture_window` answers `None` |
| `xdg_activation_v1` (the runner's) | 1 | `Activation` | `Activation::request` answers `None` |
| `infinity_window_style_v1` (the runner's) | 4, 5, 8 | `set_cards` (version 4) for [card layers](#run_layer-and-layerspec); `set_desktop` (version 5), used by `malaysia-wallpaper`; `set_plates` (version 8), which the Dock sends for its clear icons | No frames or plates |

`infinity_window_manager_v1` and `infinity_session_v1` come from the `shell`
feature of `infinity-protocol`. A missing `zwlr_layer_shell_v1` or
`ext_session_lock_manager_v1` returns
[`RunError::MissingGlobal`](../barrierkit/08-windows.md#errors).

## `run_layer` and `LayerSpec`

`run_layer(app: AppComponent, spec: LayerSpec) -> Result<(), RunError>`
creates one `zwlr_layer_surface_v1` and draws `app` in it. It returns `Ok`
when the compositor closes the surface.

| `LayerSpec` field | Type | Meaning |
|---|---|---|
| `namespace` | `String` | The layer namespace the compositor sees |
| `layer` | `Layer` | `Background`, `Bottom`, `Top` (default) or `Overlay` |
| `anchors` | `Anchors` | Edges the surface sticks to |
| `exclusive_zone` | `i32` | Space reserved on the anchored edge, in logical pixels |
| `keyboard` | `KeyboardInteractivity` | `None` (default), `Exclusive` or `OnDemand` |
| `size` | `Option<(u32, u32)>` | Logical size; `None` or a 0 lets the compositor choose |

`Anchors` has four `bool` fields: `top`, `bottom`, `left`, `right`. Two
constants cover the bars:

| Constant | Edges |
|---|---|
| `Anchors::TOP` | top, left, right |
| `Anchors::BOTTOM` | bottom, left, right |

A width of 0 needs `left` and `right`; a height of 0 needs `top` and
`bottom`. Otherwise `run_layer` returns `LayerSize` before it connects.
Popups of a layer surface work as in a window
([8. Windows](../barrierkit/08-windows.md#popups)).

A running component changes its surface with
`set_layer_geometry(LayerGeometry { anchors, exclusive_zone, size, keyboard })`.
The runner sends the new anchors, exclusive zone, size and keyboard mode and
commits; the compositor
answers with a configure, and the next frame has the new size. A geometry with
an unanchored 0 is ignored with a warning. Malaysia's notification banners
grow with their content this way, the panel takes the full height, and the
menu bar changes its height and the space it reserves:

```rust
set_layer_geometry(LayerGeometry {
    anchors: Anchors { top: true, right: true, bottom: true, left: false },
    exclusive_zone: 0,
    size: (width, 0),
    keyboard: KeyboardInteractivity::OnDemand,
});
```

A layer surface that draws cards, as Malaysia's notifications do, can ask
the compositor for the window frame around each of them with
`set_card_frames(CardFrames { radius, cards })`: `cards` are `(x, y, width,
height)` in surface-local logical pixels. The runner sends them through
`infinity_window_style_v1.set_cards` (version 4); the compositor draws the
window shadow below the surface and the inner and outer borders above it,
around a continuous corner of `radius`. The app draws each card with
`style::continuous_corner(radius)` so the shapes meet. An empty list removes
the frames.

A layer surface whose icons have a plate drawn by the compositor, as the Dock's
in the clear style, sends them with `set_plate_frames(PlateFrames { corner,
plates })` (`infinity_window_style_v1.set_plates`, version 8): `plates` are
`(x, y, width, height)` in surface-local logical pixels, and `corner` is the
radius of their continuous corner in thousandths of each plate's shorter side
(`ICON_PLATE.corner`), so icons that magnify keep their shape. The compositor
draws each as glass under the surface, and the app draws only the logo, with
`Image::glass_plate`. Only the Dock's layer surface has its plates drawn.

## `run_lock` and `SessionUnlock`

`run_lock(app: AppComponent) -> Result<(), RunError>` locks the session with
`ext_session_lock_v1` and draws `app` on one surface per output. An output
that appears later gets its own surface.

Inside `app`, `SessionUnlock::get()` returns the unlock handle.
`SessionUnlock::unlock()` ends the lock and makes `run_lock` return `Ok`.
An `unlock()` before the compositor confirms the lock waits for that
confirmation. `SessionUnlock::locked()` turns `true` once the compositor has
confirmed the lock, which Malaysia does after the lock screen's first frame is
on screen (Malaysia ADR 0074), so the app can start its entry animation then. If the compositor refuses the lock, the process prints an error
and exits with status 1.

`SessionUnlock::keyboard_layout()` returns the name of the active keyboard
layout, as the compositor's keymap gives it (for example «Spanish (Latin
American)»). It follows `wl_keyboard.keymap` and the layout index of
`wl_keyboard.modifiers`, which only the focused surface receives; while
locked, that is the lock surface.

## Screen lock and blur region

`use_screen_lock() -> State<ScreenLock>` follows `infinity_session_v1`. It is
`ScreenLock::Locking(duration)` from the moment a client asks to lock the
session, and `ScreenLock::Unlocked` again once the session is unlocked. The
menu bar uses it to close up into the lock screen's status row while the
compositor fades everything else out (Malaysia ADR 0074). A zero `duration`
means no transition.

`set_blur_region(BlurRegion { rects })` asks the compositor to blur what is
behind the calling surface inside `rects`, in surface coordinates, with
`ext_background_effect_v1`. An empty list removes the blur. Send it again
whenever the rectangles move; the kit commits the surface with it.

`set_input_region(InputRegion { rects })` limits where the calling surface
takes the pointer to `rects`, in surface coordinates (`wl_surface.set_input_region`).
Outside them, clicks reach the windows below. Malaysia's Dock leaves room
above its bar for magnified icons and takes the pointer only on the bar,
or on a strip at the screen edge while it is hidden.

## `Toplevels`

`Toplevels::get()` returns the list of windows of every app. It is a root
context in `run_layer` and `run_lock`.

| Field | Type | Holds |
|---|---|---|
| `list` | `State<Vec<ToplevelInfo>>` | The windows, in arrival order |
| `focus_bar` | `State<FocusBar>` | The bar over the shown space: `None` on a desktop, `Hidden` or `Revealed` over a focus space (Malaysia ADR 0051) |
| `desktop` | `State<Option<u32>>` | The number of the shown desktop, from 1; `None` over a focus space or without Infinity v8 |

| `ToplevelInfo` field | Type | Meaning |
|---|---|---|
| `id` | `u64` | Stable id for the window's life; every action takes it |
| `title`, `app_id` | `String` | As the app sets them |
| `active`, `maximized`, `minimized` | `bool` | Window state |
| `restorable` | `bool` | `restore` has a saved rectangle |
| `main_menu` | `Option<MainMenuAddress>` | The window's dbusmenu address ([8](../barrierkit/08-windows.md#menu-address)) |
| `gtk_menus` | `Option<GtkMenusAddress>` | The window's GTK menu address |

`GtkMenusAddress` holds `unique_bus_name`, `application_id`, `menubar_path`,
`app_menu_path`, `application_object_path` and `window_object_path`. It
exists only when the bus name and one of the two menu paths are set.

`FocusBar::over_focus()` is `true` in a focus space.

With `ext_foreign_toplevel_list_v1` alone, the list has only `id`, `title`
and `app_id`, and no action works. The actions:

| Method | Needs | Does |
|---|---|---|
| `activate(id)` | A seat | Focuses the window |
| `close(id)` | — | Asks the window to close |
| `minimize(id)`, `unset_minimized(id)` | — | Minimizes or restores |
| `set_maximized(id, bool)` | — | Maximizes or unmaximizes |
| `arrange(id, InfinityLayout)` | Infinity | Places the window; the first arrange saves its rectangle |
| `arrange_pair(InfinityPairLayout)` | Infinity | Tiles the active and the previous window |
| `arrange_quarters()` | Infinity | Tiles the four most recent windows |
| `restore(id)` | Infinity | Goes back to the saved rectangle |
| `set_fullscreen(id, bool)` | Infinity | Enters or leaves full screen |
| `force_quit(id)` | Infinity v4 | The compositor kills the window's process |
| `send_shortcut(id, keys)` | Infinity v6 | Types a chord such as `"Ctrl+Shift+S"` into the window |

`set_minimize_target(MinimizeTarget { window, rect })` tells the compositor
where the window minimizes to: `rect` is `(x, y, width, height)` in the
calling surface's coordinates (`zwlr_foreign_toplevel_handle_v1.set_rectangle`).
A width or height of 0 removes it. Malaysia's Dock sends the slot of each
window's tile, and the compositor runs the minimize effect toward it.

`capture_window(window, done)` copies a window's pixels once with
`ext-image-copy-capture-v1` and calls `done` with a `WindowImage { width,
height, rgba }`, or `None` if the compositor refuses. The kit binds a second
`ext_foreign_toplevel_list_v1` for the capture source and matches its
windows by app id and title when the list comes through
`zwlr_foreign_toplevel_management_v1`. Malaysia's Dock draws a minimized
window's tile with it.

`InfinityLayout`: `Fill`, `Center`, `Left`, `Right`, `Top`, `Bottom`,
`TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`.
`InfinityPairLayout`: `SideBySide`, `SideBySideReversed`, `Stacked`,
`StackedReversed`. The active window takes the first half.

`InfinityWindowManagerV1` is the generated proxy of
`infinity_window_manager_v1`. The shell does not call it directly;
`Toplevels` does. The protocol is
[ADR 0039](../decisions.md).

## `Activation`

`Activation::get()` returns the activation handle.
`Activation::request(callback)` asks the compositor for an `xdg_activation_v1`
token tied to the last input event. The callback gets `Some(token)`, or
`None` when there is no activation global, no seat, or no input yet. The
shell hands the token to the app it launches, so that app may take the focus.

## Status items

`use_status_items()` is a hook. It returns `(State<Vec<TrayItem>>,
StatusHost)` and starts a thread that hosts the
[StatusNotifierItem](https://specifications.freedesktop.org/status-notifier-item/latest/)
icons of other apps on the session bus:

1. It owns `org.kde.StatusNotifierWatcher` if nobody does; otherwise it follows
   the running watcher.
2. It registers as `org.kde.StatusNotifierHost-<pid>`.
3. For each item it reads the properties and the dbusmenu layout, and reads
   them again on every signal of the item or its menu. Each call waits at most
   500 ms.
4. An item goes away when it unregisters or its bus name disappears. If the
   watcher it followed disappears, it takes the watcher again.

The thread stops when the last `StatusHost` clone drops.

| `TrayItem` field | Type | From |
|---|---|---|
| `key` | `String` | Bus name and object path; the id for `StatusHost` |
| `title` | `String` | `Title` |
| `icon_name` | `String` | `IconName`, or `AttentionIconName` when `Status` is `NeedsAttention` |
| `icon_theme_path` | `String` | KDE's `IconThemePath` |
| `pixmap` | `Option<Pixmap>` | The largest valid `IconPixmap` (or `AttentionIconPixmap`) |
| `menu` | `Vec<MenuItem>` | The dbusmenu layout ([7](../barrierkit/07-menus-and-popovers.md#menu-items)) |

The menu drops hidden entries and mnemonic underscores, and keeps
separators, submenus, check marks and radio marks.

| `StatusHost` method | Sends |
|---|---|
| `activate(key)` | `Activate(0, 0)` to the item |
| `secondary_activate(key)` | `SecondaryActivate(0, 0)` |
| `click(key, id)` | A dbusmenu `clicked` event for the menu entry with action `id` |

`Pixmap` is `WindowImage`: `width`, `height` and `rgba` (straight alpha,
converted from the wire's ARGB). `Image::rgba(width, height, rgba)` draws it
([Images and template rendering](../barrierkit/04-appearance-and-tokens.md#images-and-template-rendering)).

## Shell theme

| Function | What it does |
|---|---|
| `use_shell_theme()` | Hook. Gives Freya's own components a theme built from the resolved shell colors ([`Resolved::shell`](ctshell.md#the-shell-colors-of-resolved)), and updates it when the appearance changes. Its shadow takes `THEME_SHADOW_OPACITY`. |
| `for_scheme(dark) -> Theme` | Freya theme for the default light or dark appearance. |

Next: [ctshell](ctshell.md).
