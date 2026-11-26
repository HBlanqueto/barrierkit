# 18. Platform

**Goal:** know what the runner does under a window, and which Wayland
protocols and desktop services it uses.
**Scope:** from the call to `run_window` to the frame on screen. The public
window API (`run_window`, `WindowOptions`, `WindowControls`, popups, timers,
`RunError`) is in [8. Windows](08-windows.md). The desktop services an app
calls are in [13. App](13-app.md). Shell surfaces are
not part of the kit.
**Reader:** whoever works inside the kit, or debugs an app on a compositor
other than Malaysia.
**Input:** an app that runs a window ([2. First app](02-first-app.md)).
**Output:** the knowledge to tell why a protocol or a frame is missing.

## Steps

1. Read [The runner](#the-runner) and [The frame](#the-frame) to follow a
   window from `run_window` to the screen.
2. When a feature is missing, find its global in
   [Wayland protocols](#wayland-protocols) or its bus name in
   [Desktop services](#desktop-services).
3. On a Mac, read [The macOS runner](#the-macos-runner) and
   [Build and check on macOS](#build-and-check-on-macos).
4. To compare both systems, read
   [Who provides what on each platform](#who-provides-what-on-each-platform).

## The runner

`run_window` and a shell's layer and lock runners share one runner
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
| Appearance watch (inotify) | `appearance.domain` or `wallpaper.state` changes | Resolves the appearance again and redraws every surface; `load_appearance` and `load_dominant` parse a file once until its inode, modification time or length changes, and always parse one changed in the last second |
| The same watch | Another `.domain` file changes in the config dir or in an app folder that `use_defaults` reads | Bumps the defaults revision when the app reads that file with `use_defaults`, or calls `use_watched`; the title bar reads `window.domain` |
| Settings portal ping | `org.freedesktop.appearance` changes `color-scheme`, `contrast` or `reduced-motion`; not with the `shell` feature | Resolves the appearance again and redraws every surface |
| Idle timer, every 10 s | Always | Releases Skia GPU resources unused for 10 s |

The loop ends when the last main window closes (utility windows do not
count), the compositor closes a layer surface, the lock is released, or the
Vulkan device is lost again after two recreations without a presented frame.
A swapchain image that is not ready within one second counts as a lost device. A lost device is destroyed and recreated with every surface; a presented
frame resets the count. Debug builds fake one loss after `n` draws with
`BARRIERKIT_LOSE_DEVICE_AT=n`.

## The frame

A surface draws only when something changed. A change requests a frame
callback; the frame callback steps the animations, so the frame shows them at
its own time, and draws. It requests the next frame callback before it
presents: the request reaches the compositor in the commit of the buffer.
Mutter answers a request that arrives in a later, empty commit one refresh
late, which halves the frame rate. One draw does this:

1. Picks the background: the toplevel background of the resolved appearance
   for windows, `HUD_BACKGROUND` for a HUD, transparent for layer, lock and
   popup surfaces.
2. Acquires the next swapchain image.
3. Lays out the tree with `torin` and settles the `on_sized` reactions, so a
   live resize paints the new layout.
4. Sends the sidebar glass panels (a `behind_window` visual effect of the
   `sidebar` material) and declares an opaque
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
| `wl_output` | sctk | No | Outputs | — |
| `zxdg_decoration_manager_v1` | 1 | No | The kit asks for client-side decoration; a server-side answer turns off edge resizing | Client-side |
| `wp_viewporter` + `wp_fractional_scale_manager_v1` | 1 | No | [Fractional scale](#scale) | Integer scale |
| `wp_cursor_shape_manager_v1` | sctk | No | Per-component cursor shapes | The compositor's default cursor |
| `xdg_activation_v1` | 1 | No | Raising a window; giving the keyboard to a panel | The window stays where it is |
| `xdg_wm_dialog_v1` | 1 | No | Marks a sheet window modal over its parent | The compositor does not know the sheet is modal |
| `wl_data_device_manager` | sctk | No | Clipboard and drag and drop | No clipboard |
| `zwp_primary_selection_device_manager_v1` | sctk | No | The primary selection (middle-click paste) | No primary selection |
| `zwp_text_input_manager_v3` | 1 | No | Input methods: preedit and commit | Keys only |
| `org_kde_kwin_appmenu_manager` | 1 | No | Announces a window's D-Bus menu for the global menu bar | No global menu |
| `limitless_focus_manager_v1` | 1–5 | No | Focus mode: when the bar comes down; from version 2, `set_overlay` and the bar height; from 3, the wallpaper behind the window; from 4, the desktop it is on (bk-shell reads it); from 5, how light the glass under a popup is, which sets the ink of its menu (`theme::over_glass`) | Focus mode notices, the wallpaper tint and the desktop are missing |
| `limitless_window_style_v1` | 1–9 | No | `set_bar` with the `Titlebar`'s bar; `set_popover` (v2) for popups; `set_panel` (v3) for utility windows; versions 4, 5 and 8 carry shell requests. Version 7 adds `set_popover_shape`, which a `PopoverShape` popup sends. Version 9 adds `set_knobs`, which `Slider`, `LevelSlider` and `Switch` send for the knob they hold; the kit binds at the lower of what the compositor offers and 9 ([ADR 0091](../decisions.md)) | The compositor guesses the window style |
| `limitless_effects_v1` | 1 | No | Objects for glass over the window's own content and behind it: `Sidebar` keeps a `behind_window` visual effect of the `sidebar` material; a chrome keeps a `within_window` visual effect or a glass effect (with a container when its glasses merge) and a scroll edge effect. Destroying an object removes the effect ([design](../research/limitless-effects.md), [ADR 0098](../decisions.md), [ADR 0100](../decisions.md)) | The sidebar and chromes stay a flat fill the kit draws |

`sctk` means the version range smithay-client-toolkit 0.20 binds. The
`limitless_*` protocols are Malaysia's private protocols, in
`crates/limitless/protocols/`. They carry a fraction as thousandths in a
`u32` (`ctdesign::THOUSANDTHS`). When an optional global is missing,
the runner logs it at `info` level and continues.

A held `Slider`, `LevelSlider` or `Switch` knob is glass the compositor draws above
the surface (`limitless_window_style_v1.set_knobs`, version 9). The control sends the
knob's rectangle in surface-local logical pixels, its `Knob` kind and its tint before each
commit, draws its track with no knob, and the lens refracts the pixels the app
committed there. The compositor adds the knob's border and its shadow (`KNOB_HELD_SHADOW`, `KNOB_HELD_SHADOW_DARK` in the dark scheme).
Without version 9 the knob stays the opaque `knob` paint: the kit draws no glass of its own.
On macOS the lens is a native glass view ([chapter 15](15-backdrops-and-media.md#on-macos)).

## Input and focus

- Keys go to the surface with keyboard focus, or to the first surface.
- Before a key reaches the tree, the runner matches it against the window's
  `MainMenu` shortcuts. A utility window uses its parent's menu. Edit
  shortcuts inside a focused text field stay with the field.
- `Escape` closes a utility window, unless a text field has focus. A sheet gets
  the key, as on macOS: an `Alert` answers it with its cancel button.
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
**Scope:** a window of a compositor that offers `limitless_window_style_v1` at
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
   and swapchain, and keeps a `within_window` visual effect (or a glass effect)
   and a scroll edge effect for it, changed before its buffer and only when they
   changed.
4. Draws the chrome nodes on the sub-surface's canvas, shifted by the strip's
   origin, and everything else on the window's, then presents the sub-surfaces
   and the window last. The window's commit applies theirs.

A chrome that cannot be opened is dropped and its glass stays transparent over
the content. A lost Vulkan device closes every chrome; the next draw opens them
again.

### Try it under Malaysia

Needs the owner's Linux machine with a GPU and Vulkan, a Malaysia that serves
`limitless_effects_v1` (the objects of the effects design, not the requests of
session M2), and the kit checked out beside it as `../barrierkit`. The VM
cannot run it: it has no display.

1. Terminal 1, in Malaysia: `nix develop`, then `./dev.sh kit-local`,
   `./dev.sh build` and `./dev.sh lab`. Leave it running.
2. Terminal 2, in the kit: `nix develop`. The recipes install no logger: add
   `bk::init_tracing();` as the first line of `main` in
   `crates/bk/examples/glass_window.rs`, and do not commit it. Read the socket of
   the session and open the window:

   ```sh
   socket=$(cd ../malaysia && ./dev.sh log | sed -n 's/.*listening on the Wayland socket socket=\(wayland-[0-9]*\).*/\1/p' | tail -n 1)
   WAYLAND_DISPLAY=$socket RUST_LOG=bk::chrome=debug cargo run -p bk --example glass_window
   ```

3. Look at the window, the recipe [`glass_window`](../../crates/bk/examples/glass_window.rs):
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
**Output:** a window with the system frame or the kit's own, and the kit's content.

`platform/macos/` replaces the Wayland runner. It draws with Skia on Metal into
a `CAMetalLayer`, and wakes the kit's `calloop` pings from AppKit's run loop.
A window draws at once when it was idle, and then at most once per refresh of
its display, driven by a `CADisplayLink` that pauses when nothing is due
(macOS 14 or later). A resize draws at once.

| Works | Missing |
|---|---|
| An `NSWindow` with the system title bar and traffic lights (`DecorationResponse::Server`), min and max size, title; `WindowStyle::HiddenTitleBar` keeps the native lights and the kit draws the bar beside them (`Hybrid`); `WindowStyle::Plain` is a borderless window with shadow and the kit's `Titlebar` and `TrafficLights` (`Client`), moved and resized by AppKit ([8](08-windows.md#window-style)) | Utility windows as panels, window tabs, the menu bar over a `Plain` window in full screen |
| Popups (menus, pop-up buttons, tooltips, popovers) as child panels with a native menu, popover or tooltip material, placed by `xdg_positioner`'s rules; a grabbing popup takes the keyboard and closes when it loses it. A `PopoverShape` popup has the arrow of its shape on the edge that faces the anchor (`platform/macos/arrow.rs`, [7](07-menus-and-popovers.md#other-popover-shapes)) | Glass drawn under other surfaces, continuous corners on a popover with an arrow |
| Sheets with `beginSheet`, utility windows as child windows | Dropping text or images onto a window |
| Input methods and dead keys through `NSTextInputClient`: the committed text reaches the field as the key that types it, the composing text goes to `WindowControls::ime_preedit`, `set_ime_cursor_area` places the candidates | A CJK input method and its candidate window (not run) |
| Mouse, scroll, keyboard (Command acts as Ctrl and Control as Super, as the menu draws them), cursors | |
| The menu bar from the key window's `MainMenu`, after the standard app menu (bundle name, Settings…, Services): Command key equivalents for the Ctrl shortcuts, checkmarks, disabled items, patched when the menu changes. A key that edits a focused text field goes to the field, not to the menu; so does an Edit item. Window and Help are AppKit's `windowsMenu` and `helpMenu`, and the window items are AppKit's own ([13](13-app.md#standard-menus)) | Palettes, search fields, icons and badges of a menu item; `Header` is a disabled item; the About panel of a bundled app (not run) |
| Live resize, scale from `backingScaleFactor` (not run at 2x yet), close, quit with Command+Q | |
| `portal::open_files`, `save_file` and `open_uri` as `NSOpenPanel`, `NSSavePanel` and `NSWorkspace` (`platform/macos/panels.rs`) | |
| `secrets::Secrets` as the login Keychain, generic passwords (`platform/macos/keychain.rs`) | |
| `launch` as a lock and a socket in `$TMPDIR`; a second run activates the first (`platform/macos/instance.rs`) | `ActivateAction`, files to open |
| `WindowOptions::status_item` as an `NSStatusItem` with the items as its menu (`platform/macos/status.rs`) | A status item that changes after the window opens |
| `notifications::post` and `portal::notify` as `UNUserNotificationCenter` banners, for a bundled app; the click callbacks of the last 32 posts wait, the older ones are dropped (`platform/macos/notifications.rs`) | A banner shown on screen (not run: the system refused the permission of a test bundle), action buttons |
| `Timers`, `WakeHandle`, the clipboard (typed: text, HTML, images, files), dragging typed data out | `ShapeStyle::Material` (the tint and the opaque solid) |
| The toolbar glass of a `ToolGroup` as an `NSGlassEffectView` on macOS 26 and later (the opaque fill below) | |
| AppKit's own title bar under a unified `Titlebar` that no `ScrollView` reaches, with the bar row in a titlebar accessory, on macOS 26 and later ([15](15-backdrops-and-media.md#on-macos)) | Per column of a split view |
| The glass of a toolbar `SegmentedControl`, `ToolbarSearch` field and `Button`, and of `ButtonStyle::glass` and `glass_prominent`, as an `NSGlassEffectView` each on macOS 26 and later, the prominent ones with the accent as `tintColor` ([15](15-backdrops-and-media.md#on-macos)) | SwiftUI's lens over a pressed segment, its press glow on content buttons |
| `glass_effect(Glass, Shape)` on any view as an `NSGlassEffectView` behind it on macOS 26 and later, with the view drawn in a Metal chrome above; below 26 or under Reduce Transparency the kit's own blur or opaque solid. Up to 8 chromes per window here, 3 on Linux; one radius and one variant per chrome ([15](15-backdrops-and-media.md#glass-on-any-view)) | `tint`, `interactive`, `glassEffectUnion`, a shape per corner |
| The sidebar glass, `Material`, `TopGlass` and the glass bars as `NSVisualEffectView`s with a Metal chrome above them ([15](15-backdrops-and-media.md#on-macos)) | |

The window and its popups take the system's light or dark, live ([The system's light or dark](04-appearance-and-tokens.md#the-systems-light-or-dark)). Reduce Transparency, Increase Contrast and Reduce Motion are the system's: they add to the file's (`platform/macos/system.rs`) at launch and each time the system changes them. The `shell` feature builds on macOS but adds nothing there: `bk::extension` is empty.

### Build and check on macOS

**Goal:** build the kit and run `./dev.sh check` on a Mac.
**Reader:** whoever works on `platform/macos/`.
**Input:** the repository, Xcode's command line tools and `rustup`; Nix is not needed (the flake lists Linux systems only).
**Output:** the same checks as on Linux, minus what Linux alone can run.

1. Nothing to install by hand: `rust-toolchain.toml` makes `rustup` fetch the stable toolchain with `rustfmt` and `clippy` on the first `cargo` run.
2. Run `./dev.sh check` from the repository root. It does not ask for `nix develop` on macOS.
3. It runs `cargo fmt --check`, clippy with `-D warnings -A dead_code` (window commands and the menu handle carry items only the Linux backend reads; `ctdesign`, `ctappearance` and `limitless` run again without `-A dead_code`), the unit tests, and the scripts of `scripts/` (`shellcheck` when installed), with the `shell` and `tracy` features.
4. Optional: `./dev.sh macos-smoke` opens every recipe of `crates/bk/examples` as an app and checks it ([Smoke test](#smoke-test-on-macos)).

### Smoke test on macOS

**Goal:** know that every recipe still opens a window on macOS.
**Reader:** whoever changes `platform/macos/` or a recipe.
**Input:** a Mac in a logged-in, unlocked session; `scripts/macos-smoke.sh [recipe...]` (`./dev.sh macos-smoke`); `SMOKE_SECONDS` (default 3) sets how long each recipe stays open; `CAPTURE_DIR`, optional, names a folder for a PNG of each window.
**Output:** one `PASS name` or `FAIL name: reason` line per recipe, with up to three panic or error lines of a failed one's log, a last `<n> failed` line, and the exit status 1 when any failed.

1. The script opens real windows, and captures them when `CAPTURE_DIR` is set. Nothing serializes it: run one screen-driving script at a time.
2. It builds the examples of the recipes it will open, wraps each one in an ad-hoc signed app bundle in a temporary directory and opens it with `open -n`.
3. It waits up to ten seconds for a window of that process on the screen (the CoreGraphics window list, no permission needed), then waits `SMOKE_SECONDS`.
4. It fails the recipe when no window appeared, when the process exited, or when the window went away; otherwise it ends the process.

### Appearance matrix on macOS

**Goal:** know how far each surface is from SwiftUI or AppKit in light and dark, with blur and with Reduce Transparency on.
**Reader:** whoever changes a material, a glass or a popup.
**Input:** the SwiftUI probes in `~/src/probes` (or `BARRIERKIT_PROBES`), numpy and Pillow, and `cargo build -p bk --examples`.
**Output:** per-element differences and their ceilings.

1. Run `scripts/appearance-matrix.py --transparency blur`.
2. With the system's Reduce Transparency on, run it with `--transparency rt`; the script never changes the setting.
3. It compares six surfaces, each a kit example beside its probe: `notes_sidebar` as `Plain` and as `HiddenTitleBar`, `glass_toolbar`, `glass_container`, and the popover and the sheet of `popups`. The cells, numbers and causes are in [the matrix](../research/appearance-matrix.md).

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

On macOS each window of the runner creates an `accesskit_macos`
`SubclassingAdapter` on its content view (`platform/macos/accessibility.rs`).
It is lazy: the runner builds the tree only after VoiceOver or the AX API first
asks, and answers that first query with the whole tree. A client then reads the
role, title, value, help, identifier, frame and children of each node, and
`AXPress` clicks it.

| Screen reader asks | The runner does |
|---|---|
| The tree | Builds it and sends it whole |
| `Focus` on a node | Moves the focus to that node |
| `Click` (`AXPress`) on a node | Moves the focus to that node and presses `Enter` on it |

On both systems, a tree whose views use `accessibility_element` or a text field
is sent whole on every change, because the kit folds subtrees and names the input node
([Accessibility modifiers](05-controls.md#accessibility-modifiers)). The macOS label is `AXTitle`, not `AXDescription`.

## Desktop services

The kit shares one session-bus connection per process, opens one more per
status item, and gives every method call 25 s, as libdbus and GDBus do.

| Service | Bus name | Used by |
|---|---|---|
| Settings portal | `org.freedesktop.portal.Desktop` (`Settings`, namespace `org.freedesktop.appearance`) | The system's light or dark, contrast and reduced motion, without the `shell` feature ([4](04-appearance-and-tokens.md#the-systems-light-or-dark)) |
| Notifications | `org.freedesktop.Notifications` | `notifications` ([13](13-app.md#notifications)) |
| Secret Service | `org.freedesktop.secrets` | `secrets` ([13](13-app.md#secrets)) |
| Desktop portal | `org.freedesktop.portal.Desktop` | `portal`: files, URIs, notifications ([13](13-app.md#portals)) |
| Single instance | `org.freedesktop.Application` | `Instance` ([13](13-app.md#single-instance)) |
| Global menu | `com.canonical.dbusmenu` | A window with a `MainMenu` ([13](13-app.md#menu-bar-menu)) |
| Status icon | `org.kde.StatusNotifierItem-<pid>-<n>` | `StatusItem` ([13](13-app.md#status-item)) |

## Who provides what on each platform

`platform/mod.rs` imports `linux` or `macos` as `sys` and re-exports the
same names from either: `app_folders`, `appearance`, `clipboard`, `dominant`, `instance`, `keychain`,
`menu_changed`, `notifications`, `panels`, `pasteboard`, `ObjectId`, `MOST_CHROMES`,
`NATIVE_LIGHTS`, `OPTICAL_SIZING` and `SYSTEM_MOMENTUM`; `run_window` and `Bridge` are reached
through `sys`. `zbus` is a Linux-only dependency: the dbusmenu and status item servers and the session bus live in `platform/linux/`, and the two types that cross the API are aliases (`keychain::{ItemPath, BusError}`, `instance::Error`). Each name has one signature and two
bodies. Shared code, including the shared files of `platform/`, calls
`crate::platform::<name>` and holds no `cfg(target_os)`:
`scripts/check-platform-cfg.sh` fails on one outside `platform/mod.rs`,
`platform/linux/`, `platform/macos/` and
`crates/ctappearance/`.

One API in the kit, one provider per system. The names are SwiftUI's and
AppKit's on both; the body is not shared. Read a row left to right: what the
app calls, who answers on macOS, who answers on Linux.

- **Kit:** code in `bk`, the same on both systems.
- **macOS:** AppKit, Core Animation, Metal and the Security and UserNotifications frameworks, in `crates/bk/src/platform/macos/`.
- **Linux (Malaysia):** Wayland protocols and D-Bus services; what the compositor does, it does through `limitless` ([ADR 0091](../decisions.md)). Malaysia served `limitless_window_style_v1` version 10 on 2026-11-22 and pinned the kit at `ccf65557`.

| Capability | macOS | Linux (Malaysia) | State |
|---|---|---|---|
| Window, popup, popover | `NSWindow` and child panels (`frame.rs`, `popups.rs`) | `xdg_toplevel` and `xdg_popup` on sctk; `set_popover` and `set_popover_shape` style the popup | both |
| Blur, glass, scroll edge | Core Animation layers (`blur.rs`) | The compositor: the objects of `limitless_effects_v1` | both; where the system serves no glass the client draws a flat fill |
| Traffic lights, title bar | AppKit's own; under `WindowStyle::Plain` the kit draws them ([Traffic lights](09-titlebars.md#traffic-lights)), the standard look, gray with the graphite accent | The kit draws them; the `accent` and `wallpaper` tints are Linux-only; `set_bar` styles the bar | both |
| Frame pacing | One `CADisplayLink` per window | The frame callback of the surface ([The frame](#the-frame)) | both |
| Full screen (the green light) | `toggleFullScreen`, with the menu bar and the Dock hidden for a `Plain` window; the delegate tells `WindowStatus::fullscreen` ([recipe 39](../../.claude/skills/barrierkit-app/recipes.md#39-full-screen-with-each-window-style)) | `xdg_toplevel.set_fullscreen`; the compositor draws the bar | both |
| Clipboard | `macos/clipboard.rs` (text) and `macos/pasteboard.rs` (typed: text, HTML, PNG and TIFF, file URLs, private boards) on `NSPasteboard` | `wl_data_device`, and `zwp_primary_selection_v1` for the primary selection; `linux/pasteboard.rs` holds text only, so images, files and named boards report false | both; typed on macOS only |
| Drag and drop | Out: an `NSDraggingSession` from the content view with `NSPasteboardItem`s (`macos/view.rs`, `macos/pasteboard.rs`); in: the content view is the `NSDraggingDestination` for file URLs | `wl_data_device`; out offers text, `text/html`, `image/png` and `text/uri-list`; in accepts only `text/uri-list` | Both ways, files only inwards; macOS not run with a real drag |
| Text input | `NSTextInputClient` (`macos/view.rs`): committed and composing text, the candidates at `set_ime_cursor_area`; a CJK input method not run | `zwp_text_input_v3` (`ime.rs`) | both, macOS partial |
| Accessibility | `accesskit_macos` on the content view (`macos/accessibility.rs`): VoiceOver and the AX API ([Accessibility](#accessibility)) | AT-SPI through `accesskit_unix` ([Accessibility](#accessibility)) | both |
| Menus | `NSMenu` (`menu.rs`) | `com.canonical.dbusmenu`, announced with `org_kde_kwin_appmenu_manager`; the shell reads it from the `app_menu` event of the shell protocol `limitless_window_manager_v1` (`limitless-shell`) | both |
| Status item | `NSStatusItem` (`status.rs`) | `StatusNotifierItem` | both |
| File dialogs | `NSOpenPanel` and `NSSavePanel` as a sheet of the window, with extensions and a start folder (`macos/panels.rs`) | The `FileChooser` portal with the extensions as `filters` and the start folder as `current_folder`; no parent window, which needs `xdg-foreign` (`linux/panels.rs`, `linux/portal.rs`) | both, Linux partial |
| Documents | Dirty dot and `representedURL`, the close veto (`windowShouldClose:`), the "Save changes?" `NSAlert` sheet, Cmd-Q and `application(_:openURLs:)` (`macos/panels.rs`, `macos/delegate.rs`) | The close request and `WindowControls::close()` are vetoed; the kit's `Alert` in a sheet window asks with the same buttons; quit closes the first edited window first; `org.freedesktop.Application.Open` hands each URI to `on_open_url`. No dirty dot (`linux/panels.rs`, `linux/window.rs`) | both |
| Notifications | `UserNotifications` (`macos/notifications.rs`) | `org.freedesktop.Notifications` and the portal (`linux/notifications.rs`) | both |
| Secrets | Keychain (`security-framework`, `macos/keychain.rs`) | Secret Service (`linux/keychain.rs`) | both |
| Single instance | `macos/instance.rs` | `org.freedesktop.Application` (`linux/instance.rs`) | both |
| Appearance | `NSWorkspace` accessibility notifications (`system.rs`) | `.domain` files watched with inotify, and `org.freedesktop.appearance`: `color-scheme`, and `contrast` and `reduced-motion` added to the file's, without the `shell` feature (`linux/system.rs`) | both |

What the [gap document](../research/swiftui-api-gap.md#what-an-app-author-still-needs)
left out of the current round, by who would answer it:

| Gap | Kit | macOS | Linux (Malaysia) |
|---|---|---|---|
| 3 `ZStack` and `Layout`, 6 `Transition`, 15 right-to-left | Yes: Freya and Torin, the same on both | none | none |
| 4 Accessibility by SwiftUI name | The modifiers (done) | `accesskit_macos` (done) | AT-SPI (done) |
| 7 Typed drag and drop | `Transferable` and `Pasteboard` (done) | `NSPasteboard` types (done); a drop target (files only) | `wl_data_device` MIME types (offers done; a typed drop target none) |
| 11 `glassEffect` on any view | The modifier | A layer per view | A `limitless_effects_v1` object on a sub-surface per view |
| 12 Document apps | `Document`, `use_document`, `on_open_url` (done, [Documents](08-windows.md#documents-and-file-dialogs)) | `NSWindow` dirty state, `windowShouldClose:`, `NSAlert`, the app delegate (done) | The close veto, the kit's `Alert` and `org.freedesktop.Application.Open` (done); no dirty dot |
| 13 Dialogs attached to their window | `FileDialog`, `file_importer`, `file_exporter` (done) | `beginSheetModalForWindow` (done) | `xdg-foreign`, not bound |

How to keep it true:

1. A new capability gets its row in the commit that adds it.
2. The Linux cell names the request and the version that carries it. A client reads the version it bound and does not send a request above it: a client that binds 8 cannot send `set_knobs` ([ADR 0091](../decisions.md)).
3. A function that needs a new request goes into `limitless` with a `since` number first. Malaysia serves it after it bumps its pin.
4. `cfg(target_os)` stays in `platform/` ([crates/AGENTS.md](../../crates/AGENTS.md)): `platform/macos/` and `platform/linux/` hold what one system uses, and `platform/` holds what both use. The matrix names the file.

To check each row on a Linux machine after changes made on a Mac, follow [Linux verification](linux-verification.md).

## Errors

Each `RunError` variant and the runner step that raises it are in
[8. Windows](08-windows.md#errors).

A draw that fails for another reason logs one warning and retries on the
next frame.

Next: [19. Profiling](19-profiling.md).
