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
Vulkan device is lost again after two recreations without a presented frame.
A lost device is destroyed and recreated with every surface; a presented
frame resets the count. Debug builds fake one loss after `n` draws with
`BARRIERKIT_LOSE_DEVICE_AT=n`.

## The frame

A surface draws only when something changed. A change requests a frame
callback; the frame callback draws. One draw does this:

1. Picks the background: the toplevel background of the resolved appearance
   for windows, `HUD_BACKGROUND` for a HUD, transparent for layer, lock and
   popup surfaces.
2. Acquires the next swapchain image.
3. Lays out the tree with `torin` and settles the `on_sized` reactions, so a
   live resize paints the new layout.
4. Sends the sidebar glass panels (`set_backdrop` with `behind_window` and the
   `sidebar` material from version 10, `set_glass` before) and declares an opaque
   background, minus the panels, as the opaque region. They reach the compositor
   in the commit of the buffer they describe.
5. Opens, moves and closes the [chrome sub-surfaces](#glass-over-the-windows-content)
   of the window's declared glass, and acquires their swapchain images.
6. Records the tree with Skia: the chrome nodes on the canvases of their
   sub-surfaces, the rest on the window's.
7. Submits and presents, the chromes first and the window last.

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
| Device extensions | `VK_KHR_swapchain`; `VK_EXT_swapchain_maintenance1` when present fences are supported |
| Context | One Skia `DirectContext` for every surface of the process, with a 128 MB resource cache |
| Swapchain | One per surface, a [chrome sub-surface](#glass-over-the-windows-content) included. `B8G8R8A8_UNORM`, sRGB. `MAILBOX` with 3 images when available, else `FIFO` with 2. Premultiplied alpha when the surface supports it |

Frame callbacks already pace the redraws, so the runner prefers `MAILBOX`
([freya#2260](https://github.com/marc2332/freya/issues/2260)). The decision
to use Vulkan is [ADR 0056](../decisions.md).

## Wayland protocols

| Global | Version | Needed | What for | Without it |
|---|---|---|---|---|
| `wl_compositor` | sctk | Always | Surfaces | `MissingGlobal("wl_compositor")` |
| `xdg_wm_base` | sctk | Windows | Toplevels and popups | `run_window` returns `MissingGlobal("xdg_wm_base")` |
| `wl_subcompositor` | sctk | No | The [chrome sub-surfaces](#glass-over-the-windows-content) of a window with glass | The glass is drawn by the kit |
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
| `infinity_focus_manager_v1` | 1–5 | No | Focus mode: when the bar comes down; from version 2, `set_overlay` and the bar height; from 3, the wallpaper behind the window; from 4, the desktop it is on (`WindowControls::desktop`); from 5, how light the glass under a popup is, which sets the ink of its menu (`theme::over_glass`) | Focus mode notices, the wallpaper tint and the desktop are missing |
| `infinity_session_v1` | 1 | No | `locking` and `unlocked`, read by [`use_screen_lock`](#screen-lock-and-blur-region) | The menu bar never learns that the session is locking |
| `ext_background_effect_manager_v1` | 1 | No | [`set_blur_region`](#screen-lock-and-blur-region) | No blur behind the surface |
| `ext_image_copy_capture_manager_v1` + `ext_foreign_toplevel_image_capture_source_manager_v1` + `wl_shm` | 1 | No | [`capture_window`](#toplevels), in layer-shell components | `capture_window` answers `None` |
| `infinity_window_style_v1` | 1–10 | No | `set_bar` with the `Titlebar`'s bar; `set_popover` (v2) for popups; `set_panel` (v3) for utility windows; `set_cards` (v4) for card layers. Version 5 adds `set_desktop`, used by `malaysia-wallpaper`. Version 6 adds `set_glass`, which `Sidebar` sends for its glass panel ([ADR 0065](../decisions.md)); from version 10 it sends `set_backdrop` with `behind_window` instead ([ADR 0100](../decisions.md)). Version 8 adds `set_plates`, which the Dock sends for its clear icons. Version 9 adds `set_knobs`, which `Slider`, `LevelSlider` and `Switch` send for the knob they hold. Version 10 adds `set_backdrop` and `set_scroll_edge`, for glass over the window's own content: the kit binds at the lower of what the compositor offers and 10 and sends them only from 10 ([ADR 0091](../decisions.md), [ADR 0098](../decisions.md)) | The compositor guesses the window style; the glass is drawn by the kit |

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
- Touch events go to the surface where the touch started. Malaysia sends the
  pointer and touch events over a [chrome sub-surface](#glass-over-the-windows-content)
  to the window's own surface, in window coordinates, so the runner maps
  nothing; a compositor that sent them to the sub-surface would need a mapping.
- The cursor is the one the topmost hovered element declares with
  `.cursor(icon)`; a view that drags declares it on its container while the
  drag lasts. `CursorIcon` values map to `wp_cursor_shape_device_v1` shapes.

## Glass over the window's content

[ADR 0098](../decisions.md) is the decision, [the design](../research/blur-backdrop-design.md)
the plan. This section says what the runner does and how to see it.

**Goal:** draw the glass of `TopGlass` and `Material` with the compositor's
blur, over the window's own content.
**Scope:** a window of a compositor that offers `infinity_window_style_v1` at
version 10 or later. Everywhere else the kit draws the glass itself
([15](15-backdrops-and-media.md#who-draws-the-blur)).
**Reader:** whoever changes the runner, or checks the glass on a real session.
**Input:** a laid-out tree with declared glass. **Output:** one sub-surface per
container that holds a glass, and the requests that describe it.

### What the runner does

1. Binds the global at the lower of its version and 10, and `wl_subcompositor`.
   Both must be there for a window to take part.
2. After each layout, reads the tree for the glass leaves (`platform::backdrop::collect`).
   The element that holds a leaf is a chrome; chromes that contain one another
   merge; a holder with nothing beside it in the tree is skipped.
3. For each chrome opens a synchronized `wl_subsurface` above the window's
   surface at the strip's position (`set_position`), with its own `wp_viewport`
   and swapchain, and sends `set_backdrop` (`within_window`) and
   `set_scroll_edge` to it, before its buffer and only when they changed.
4. Draws the chrome nodes on the sub-surface's canvas, shifted by the strip's
   origin, and everything else on the window's, then presents the sub-surfaces
   and the window last. The window's commit applies theirs.

A chrome that cannot be opened is dropped and its glass stays transparent over
the content. A lost Vulkan device closes every chrome; the next draw opens them
again.

### Try it under Malaysia

Needs the owner's Linux machine with a GPU and Vulkan, a Malaysia that serves
`set_backdrop` and `set_scroll_edge` and advertises version 10 (the commit of
session M2), and the kit checked out beside it as `../barrierkit`. The VM
cannot run it: it has no display.

1. Terminal 1, in Malaysia: `nix develop`, then `./dev.sh kit-local`,
   `./dev.sh build` and `./dev.sh lab`. Leave it running.
2. Terminal 2, in the kit: `nix develop`. Read the socket of the session and
   open the window:

   ```sh
   socket=$(cd ../malaysia && ./dev.sh log | sed -n 's/.*listening on the Wayland socket socket=\(wayland-[0-9]*\).*/\1/p' | tail -n 1)
   WAYLAND_DISPLAY=$socket RUST_LOG=bk::chrome=debug cargo run -p bk --example glass_window
   ```

3. Look at the window, the recipe [`glass_window`](../../agents/recipes/glass_window.rs):
   - The log shows two `chrome surface opened` lines: the title bar and the
     card at the bottom right. With a Malaysia that advertises 9 it shows none
     and the bar is blurred by the kit.
   - Scroll the list. Under the title bar the colored rows are blurred and fade
     out toward the bottom of the bar; the title and the traffic lights stay
     sharp; the blur follows the scroll without a frame behind. The card shows
     the rows blurred behind its sharp label.
   - Drag the window by the bar, press the traffic lights, resize the window:
     the bar answers as before and nothing flickers.
   - Scroll with the pointer over the bar.
   - Turn on `reduce_transparency` ([4](04-appearance-and-tokens.md#the-appearance-file)):
     the bar and the card become opaque and no chrome is opened.
4. Send back the log, what looked wrong, and whether the compositor's log shows
   a protocol error.

## The macOS runner

**Goal:** know what an app does on macOS arm64 today.
**Scope:** `run_window` on `aarch64-apple-darwin`. Design and milestones:
[macOS arm64](../research/macos-arm-port.md).
**Reader:** whoever runs a kit app on a Mac.
**Input:** an app built with `cargo run -p bk --example <name>` on the Mac.
**Output:** a window with the system frame and the kit's content.

`platform/macos/` replaces the Wayland runner. It draws with Skia on Metal into
a `CAMetalLayer`, and wakes the kit's `calloop` pings from AppKit's run loop.

| Works | Missing |
|---|---|
| An `NSWindow` with the system title bar and traffic lights (`DecorationResponse::Server`), min and max size, title | Utility windows as panels, window tabs |
| Popups (menus, pop-up buttons, tooltips, popovers) as child panels with a native menu, popover or tooltip material, placed by `xdg_positioner`'s rules; a grabbing popup takes the keyboard and closes when it loses it | The popover arrow band, glass drawn under other surfaces |
| Sheets with `beginSheet`, utility windows as child windows | Text input methods, drag and drop |
| Mouse, scroll, keyboard (Command acts as Ctrl), cursors | |
| Live resize, scale from `backingScaleFactor` (not run at 2x yet), close, quit with Command+Q | A native menu bar from `MainMenu`, notifications, file panels, accessibility |
| `Timers`, `WakeHandle`, clipboard text | Blur and glass (the kit draws the opaque solid), the live system appearance |

The window and its popups take the kit's scheme (`appearance.domain`), not the system's. The `shell` feature does not build on macOS.

## Accessibility

With the `a11y` feature ([1](01-what-is-barrierkit.md#the-crates)), each
toplevel window publishes its accessibility tree over AT-SPI with
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
| Desktop portal | `org.freedesktop.portal.Desktop` | `portal`: files, URIs, notifications ([13](13-app.md#portals)) |
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
([ADR 0047](../decisions.md)).

| Global | Version | Used by |
|---|---|---|
| `zwlr_layer_shell_v1` | sctk | `run_layer` |
| `ext_session_lock_manager_v1` | sctk | `run_lock` |
| `zwlr_foreign_toplevel_manager_v1` | 3 | `Toplevels`, first choice |
| `ext_foreign_toplevel_list_v1` | 1 | `Toplevels`, only when the wlr manager is missing |
| `infinity_window_manager_v1` | 1–8 | Tiling, force quit, shortcuts, spaces, the shown desktop and menu addresses in `Toplevels` |
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

A held `Slider`, `LevelSlider` or `Switch` knob is glass the compositor draws above
the surface (`infinity_window_style_v1.set_knobs`, version 9). The control sends the
knob's rectangle in surface-local logical pixels, its `Knob` kind and its tint before each
commit, draws its track with no knob, and the lens refracts the pixels the app
committed there. The compositor adds the knob's border and its shadow (`KNOB_HELD_SHADOW`, `KNOB_HELD_SHADOW_DARK` in the dark scheme).
Without version 9 the knob stays the opaque `knob` paint: the kit draws no glass of its own.

### `run_lock` and `SessionUnlock`

`run_lock(app: AppComponent) -> Result<(), RunError>` locks the session with
`ext_session_lock_v1` and draws `app` on one surface per output. An output
that appears later gets its own surface.

Inside `app`, `SessionUnlock::get()` returns the unlock handle.
`SessionUnlock::unlock()` ends the lock and makes `run_lock` return `Ok`.
An `unlock()` before the compositor confirms the lock waits for that
confirmation. `SessionUnlock::locked()` turns `true` once the compositor has
confirmed the lock, which Malaysia does after the lock screen's first frame is
on screen (ADR 0074), so the app can start its entry animation then. If the compositor refuses the lock, the process prints an error
and exits with status 1.

`SessionUnlock::keyboard_layout()` returns the name of the active keyboard
layout, as the compositor's keymap gives it (for example «Spanish (Latin
American)»). It follows `wl_keyboard.keymap` and the layout index of
`wl_keyboard.modifiers`, which only the focused surface receives; while
locked, that is the lock surface.

### Screen lock and blur region

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

### `Toplevels`

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
| `main_menu` | `Option<MainMenuAddress>` | The window's dbusmenu address ([8](08-windows.md#menu-address)) |
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
the wire's ARGB). `Image::rgba(width, height, rgba)` draws it
([Images and template rendering](04-appearance-and-tokens.md#images-and-template-rendering)).

Next: [19. Profiling](19-profiling.md).
