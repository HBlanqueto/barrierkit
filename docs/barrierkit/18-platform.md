# 18. Platform

**Goal:** know what the runner does under a window, which Wayland protocols
and desktop services it uses, and open a shell surface with the `shell`
feature.
**Scope:** from the call to `run_window`, `shell::run_layer` or
`shell::run_lock` to the frame on screen. The public window API
(`run_window`, `WindowOptions`, `WindowControls`, popups, timers, `RunError`)
is in [8. Windows](08-windows.md). The desktop services an app calls are in
[13. App](13-app.md).
**Reader:** whoever works inside the kit, writes a shell component, or debugs
an app on a compositor other than Malaysia.
**Input:** an app that runs a window ([2. First app](02-first-app.md)).
**Output:** a layer surface (menu bar, Dock, background) or a lock screen
drawn by BarrierKit, and the knowledge to tell why a protocol or a frame is
missing.

## Steps

These steps open a shell surface, the task only this chapter covers.

1. Enable the feature in the shell's `Cargo.toml`:

   ```toml
   bk = { git = "https://github.com/HBlanqueto/barrierkit", rev = "…", features = ["shell"] }
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

## The runner

`run_window`, `run_layer` and `run_lock` share one runner
(`platform::runtime::Runtime`). It does this, in order:

1. Connects to `$WAYLAND_DISPLAY` with smithay-client-toolkit and binds the
   globals ([protocols](#wayland-protocols)).
2. Creates the Vulkan device and the Skia context ([Vulkan](#vulkan-and-skia)).
3. Creates the surface: an `xdg_toplevel`, a `zwlr_layer_surface_v1` or one
   `ext_session_lock_surface_v1` per output.
4. Runs a `calloop` loop with these sources:

| Source | Wakes up when | Then |
|---|---|---|
| Wayland | The compositor sends events | Input, configure, frame callbacks |
| Freya ping | A component changes state or asks for a redraw | Polls every surface, opens pending popups and windows, installs timers, requests frame callbacks |
| Wake ping | A `WakeHandle`, a D-Bus menu or another thread wakes the app | Runs the wake callbacks and menu actions, then polls |
| Appearance watch (inotify) | `appearance.domain` or `wallpaper.state` changes | Resolves the appearance again and redraws every surface |
| The same watch | Another `.domain` file in the config dir changes | Bumps the defaults revision, so `use_defaults` reads again |
| Idle timer, every 10 s | Always | Releases Skia GPU resources unused for 10 s |

The loop ends when the last main window closes (utility windows do not
count), the compositor closes a layer surface, the lock is released, or the
Vulkan device is lost.

## The frame

A surface draws only when something changed. A change requests a frame
callback; the frame callback draws. One draw does this:

1. Picks the background: the toplevel background of the resolved appearance
   for windows, `HUD_BACKGROUND` for a HUD, transparent for layer, lock and
   popup surfaces. An opaque background is declared as the opaque region.
2. Acquires the next swapchain image.
3. Lays out the tree with `torin` and settles the `on_sized` reactions, so a
   live resize paints the new layout.
4. Records the whole tree with Skia.
5. Submits and presents.

### The first frame

The first draw happens when the first configure arrives, without waiting for
a frame callback. An acked size must arrive with a buffer, and a compositor
may not send frame callbacks to a surface without content. If that first draw
fails, the runner recreates the swapchain and draws once more. Later frames
wait for their frame callback.

### Scale

With `wp_viewporter` and `wp_fractional_scale_manager_v1`, the buffer has the
fractional size and a viewport maps it to the logical size. Without them, the
runner uses the integer `wl_surface` buffer scale.

## Vulkan and Skia

| Step | Detail |
|---|---|
| Loader | `libvulkan`, the system loader |
| Instance | `VK_KHR_surface`, `VK_KHR_wayland_surface`; with Vulkan 1.1, also `VK_KHR_get_surface_capabilities2` and `VK_EXT_surface_maintenance1` when present |
| Device | The first GPU with a graphics queue that can present to the display. Mesa's device-select layer lists the compositor's GPU first |
| Device extensions | `VK_KHR_swapchain`; `VK_EXT_swapchain_maintenance1` when present fences are supported; the dmabuf import set with the `video` feature ([15](15-backdrops-and-media.md)) |
| Context | One Skia `DirectContext` for every surface of the process, with a 128 MB resource cache |
| Swapchain | One per surface. `B8G8R8A8_UNORM`, sRGB. `MAILBOX` with 3 images when available, else `FIFO` with 2. Premultiplied alpha when the surface supports it |

Frame callbacks already pace the redraws, so the runner prefers `MAILBOX`
([freya#2260](https://github.com/marc2332/freya/issues/2260)). The decision
to use Vulkan is [ADR 0056](../decisions/0056-barrierkit-on-vulkan.md).

## Wayland protocols

| Global | Version | Needed | What for | Without it |
|---|---|---|---|---|
| `wl_compositor` | sctk | Always | Surfaces | `MissingGlobal("wl_compositor")` |
| `xdg_wm_base` | sctk | Windows | Toplevels and popups | `run_window` returns `MissingGlobal("xdg_wm_base")` |
| `wl_seat` | sctk | No | Pointer, keyboard and touch | No input |
| `wl_output` | sctk | No | Outputs; one lock surface each | — |
| `zxdg_decoration_manager_v1` | 1 | No | The kit asks for client-side decoration; a server-side answer turns off edge resizing | Client-side |
| `wp_viewporter` + `wp_fractional_scale_manager_v1` | 1 | No | [Fractional scale](#scale) | Integer scale |
| `wp_cursor_shape_manager_v1` | sctk | No | Per-component cursor shapes | The compositor's default cursor |
| `xdg_activation_v1` | 1 | No | Raising a window; giving the keyboard to a panel | The window stays where it is |
| `wl_data_device_manager` | sctk | No | Clipboard and drag and drop | No clipboard |
| `zwp_primary_selection_device_manager_v1` | sctk | No | The primary selection (middle-click paste) | No primary selection |
| `zwp_text_input_manager_v3` | 1 | No | Input methods: preedit and commit | Keys only |
| `org_kde_kwin_appmenu_manager` | 1 | No | Announces a window's D-Bus menu for the global menu bar | No global menu |
| `infinity_focus_manager_v1` | 1–2 | No | Focus mode: when the bar comes down; from version 2, `set_overlay` and the bar height | Focus mode notices are missing |
| `infinity_window_style_v1` | 1–3 | No | `set_bar` with the `Titlebar`'s bar; `set_popover` (v2) for popups; `set_panel` (v3) for utility windows | The compositor guesses the window style |

`sctk` means the version range smithay-client-toolkit 0.20 binds. The
`infinity_*` protocols are Malaysia's private protocols, in
`crates/infinity-protocol/protocols/`. When an optional global is missing,
the runner logs it at `info` level and continues.

## Input and focus

- Keys go to the surface with keyboard focus, or to the first surface.
- Before a key reaches the tree, the runner matches it against the window's
  `MainMenu` shortcuts. A utility window uses its parent's menu. Edit
  shortcuts inside a focused text field stay with the field.
- `Escape` closes a utility window, unless a text field has focus.
- Key repeat follows the compositor's rate and delay.
- Tab and Shift+Tab move the focus to the next or previous node, leaving the
  current group; Up and Down move it inside a group (Freya's `integration`
  root). A focus move turns on keyboard
  navigation, which shows the focus ring.
- Touch events go to the surface where the touch started.
- `CursorIcon` values map to `wp_cursor_shape_device_v1` shapes.

## Accessibility

Each toplevel window publishes its accessibility tree over AT-SPI with
`accesskit_unix`. Layer surfaces, lock surfaces and popups publish none.

| Screen reader asks | The runner does |
|---|---|
| The tree | Sends the full tree on the next update |
| `Focus` on a node | Moves the focus to that node |
| `Click` on a node | Moves the focus to that node and presses `Enter` on it |

Other actions are ignored. After the first tree, each update sends only the
nodes that changed and the focus.

## Desktop services

The kit opens one session-bus connection per process and gives every method
call 25 s, as libdbus and GDBus do.

| Service | Bus name | Used by |
|---|---|---|
| Notifications | `org.freedesktop.Notifications` | `notifications` ([13](13-app.md#notifications)) |
| Secret Service | `org.freedesktop.secrets` | `secrets` ([13](13-app.md#secrets)) |
| Desktop portal | `org.freedesktop.portal.Desktop` | `portal`: files, URIs, printing, notifications ([13](13-app.md#portals)) |
| Single instance | `org.freedesktop.Application` | `Instance` ([13](13-app.md#single-instance)) |
| Global menu | `com.canonical.dbusmenu` | A window with a `MainMenu` ([13](13-app.md#menu-bar-menu)) |
| Status icon | `org.kde.StatusNotifierItem-<pid>-<n>` | `StatusItem` ([13](13-app.md#status-item)) |
| Status host | `org.kde.StatusNotifierWatcher` | [`use_status_items`](#status-items) |

## Errors

Each `RunError` variant and the runner step that raises it are in
[8. Windows](08-windows.md#errors).

A draw that fails for another reason logs one warning and retries on the
next frame.

## The `shell` feature

Only shell components enable `bk/shell`. The feature adds the
`shell` module. Its submodules are private; everything public comes from
`shell/mod.rs`.

`platform/` never imports `shell/`. The shell hooks into the runner through
one `Extension` with four hooks: provide the root contexts, a new seat, a new
output and a poll on each loop turn
([ADR 0047](../decisions/0047-barrierkit-in-layers.md)).

| Global | Version | Used by |
|---|---|---|
| `zwlr_layer_shell_v1` | sctk | `run_layer` |
| `ext_session_lock_manager_v1` | sctk | `run_lock` |
| `zwlr_foreign_toplevel_manager_v1` | 3 | `Toplevels`, first choice |
| `ext_foreign_toplevel_list_v1` | 1 | `Toplevels`, only when the wlr manager is missing |
| `infinity_window_manager_v1` | 1–6 | Tiling, force quit, shortcuts, spaces and menu addresses in `Toplevels` |
| `xdg_activation_v1` | 1 | `Activation` |

### `run_layer` and `LayerSpec`

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
([8. Windows](08-windows.md#popups)).

### `run_lock` and `SessionUnlock`

`run_lock(app: AppComponent) -> Result<(), RunError>` locks the session with
`ext_session_lock_v1` and draws `app` on one surface per output. An output
that appears later gets its own surface.

Inside `app`, `SessionUnlock::get()` returns the unlock handle.
`SessionUnlock::unlock()` ends the lock and makes `run_lock` return `Ok`.
An `unlock()` before the compositor confirms the lock waits for that
confirmation. If the compositor refuses the lock, the process prints an error
and exits with status 1.

### `Toplevels`

`Toplevels::get()` returns the list of windows of every app. It is a root
context in `run_layer` and `run_lock`.

| Field | Type | Holds |
|---|---|---|
| `list` | `State<Vec<ToplevelInfo>>` | The windows, in arrival order |
| `space` | `State<SpaceInfo>` | The shown space (Malaysia ADR 0051) |

| `ToplevelInfo` field | Type | Meaning |
|---|---|---|
| `id` | `u64` | Stable id for the window's life; every action takes it |
| `title`, `app_id` | `String` | As the app sets them |
| `active`, `maximized`, `minimized` | `bool` | Window state |
| `restorable` | `bool` | `restore` has a saved rectangle |
| `main_menu` | `Option<MainMenuAddress>` | The window's dbusmenu address ([8](08-windows.md#menu-address)) |
| `gtk_menus` | `Option<GtkMenusAddress>` | The window's GTK menu address |

`GtkMenusAddress` holds `unique_bus_name`, `application_id`, `menubar_path`,
`app_menu_path`, `application_object_path` and `window_object_path`. It
exists only when the bus name and one of the two menu paths are set.

| `SpaceInfo` field | Type | Meaning |
|---|---|---|
| `index` | `u32` | 0-based position of the shown space |
| `count` | `u32` | Number of spaces |
| `desktop` | `Option<u32>` | 1-based desktop number; `None` in a focus space |
| `focus_bar` | `FocusBar` | `None` (a desktop), `Hidden` (focus space, bar up) or `Revealed` (focus space, bar down) |

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

`InfinityLayout`: `Fill`, `Center`, `Left`, `Right`, `Top`, `Bottom`,
`TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`.
`InfinityPairLayout`: `SideBySide`, `SideBySideReversed`, `Stacked`,
`StackedReversed`. The active window takes the first half.

`InfinityWindowManagerV1` is the generated proxy of
`infinity_window_manager_v1`. The shell does not call it directly;
`Toplevels` does. The protocol is
[ADR 0039](../decisions/0039-infinity-v1-window-management-for-the-shell.md).

### `Activation`

`Activation::get()` returns the activation handle.
`Activation::request(callback)` asks the compositor for an `xdg_activation_v1`
token tied to the last input event. The callback gets `Some(token)`, or
`None` when there is no activation global, no seat, or no input yet. The
shell hands the token to the app it launches, so that app may take the focus.

### Status items

`use_status_items()` is a hook. It returns `(State<Vec<TrayItem>>,
StatusHost)` and starts a thread that hosts the
[StatusNotifierItem](https://specifications.freedesktop.org/status-notifier-item/latest/)
icons of other apps:

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
| `menu` | `Vec<MenuItem>` | The dbusmenu layout ([7](07-menus-and-popovers.md#menu-items)) |

The menu drops hidden entries and mnemonic underscores, and keeps
separators, submenus, check marks and radio marks.

| `StatusHost` method | Sends |
|---|---|
| `activate(key)` | `Activate(0, 0)` to the item |
| `secondary_activate(key)` | `SecondaryActivate(0, 0)` |
| `click(key, id)` | A dbusmenu `clicked` event for the menu entry with action `id` |

`Pixmap` has `width`, `height` and `rgba` (straight alpha, converted from
the wire's ARGB). `pixmap_image(pixmap, size)` returns an image element of
`size` × `size` logical pixels, or `None` when Skia rejects the data.

Next: [19. Profiling](19-profiling.md).
