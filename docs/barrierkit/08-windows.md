# 8. Windows

**Goal:** open a window with `run_window`, set its options, read and change
its state, open utility windows and popups, and run code later.
**Scope:** the window API of `bk::platform`: `run_window`,
`WindowOptions`, `WindowControls`, `WindowStatus`, utility windows, popups,
timers, input helpers and `RunError`. `Application` and `Scene` are in
[2. Your first app](02-first-app.md). The title bar and the traffic lights are
in [9. Titlebars](09-titlebars.md). How the runner works inside (Vulkan,
Wayland protocols) is in [18. Platform](18-platform.md).
**Reader:** anyone who builds the window of an app.
**Input:** an app that runs with `Application` ([2](02-first-app.md)).
**Output:** an app that controls its own window.

## Steps

1. Use `run_window` instead of `Application` when the window draws its own
   title bar (a sidebar, toolbar items) or the app opens more windows.
   `Application::window(..).run()` is `run_window` with a background, a
   `Titlebar` and centered content ([2](02-first-app.md#what-application-does-for-you)).
2. Describe the window with `WindowOptions` and pass the root view:

   ```rust
   use bk::prelude::*;

   fn main() -> Result<(), RunError> {
       run_window(
           || {
               rect()
                   .expanded()
                   .background(color(Palette::current().window_background))
                   .child(Titlebar::new().title(tr!("notes")))
           },
           WindowOptions::new(tr!("notes"), "notes")
               .size((760, 520))
               .min_size(Some(WINDOW_MIN_SIZE)),
       )
   }
   ```

3. Inside any component, read the state and send commands through
   `WindowControls`:

   ```rust
   use bk::platform::WindowControls;

   let controls = WindowControls::get();
   let fullscreen = controls.status.read().fullscreen;
   controls.set_title(tr!("notes-draft"));
   ```

4. Open popups, utility windows or timers from event handlers
   ([Popups](#popups), [Utility windows](#utility-windows),
   [Timers](#timers)).

## Running a window

`run_window(app, options) -> Result<(), RunError>` connects to the
compositor, opens one `xdg_toplevel` and runs the event loop until the last
main window closes. `app` is anything that converts into an `AppComponent`: a
closure that returns a view, or `AppComponent::new(component)`.

The runner provides these root contexts to every component of the window,
each read with `get()`:

| Context | For |
|---|---|
| `WindowControls` | The window's state and commands ([Window controls](#window-controls)) |
| `Popups` | Opening popups ([Popups](#popups)) |
| `Timers` | Running code later ([Timers](#timers)) |

The content of a popup gets `Popups`, `Timers` and its own
[`PopupHandle`](#popups), but no `WindowControls`. `WindowControls::get()`
panics where there is none (a popup, a test without a runner);
`try_consume_root_context::<WindowControls>()` returns `None` instead.

## Window options

`WindowOptions::new(title, app_name)` starts with the defaults below; each
method changes one field (the fields are also public).

| Method | Default | Effect |
|---|---|---|
| `new(title, app_name)` | — | Visible title and short, stable name |
| `.app_id(id)` | `io.github.hblanqueto.malaysia.<app_name>` | The Wayland `app_id`. [`titlebar::app_id(name)`](09-titlebars.md#helpers) builds the default |
| `.size((width, height))` | `WINDOW_DEFAULT_SIZE` (640 × 420) | Initial size. A size saved on close wins, clamped to the limits |
| `.min_size(Some((w, h)))`, `.max_size(..)` | `None` | Limits. Equal limits make the window fixed-size (`WindowStatus::fixed`) |
| `.menu(MainMenu)` | none | The window's menu bar menu ([13. App](13-app.md)) |
| `.status_item(StatusItem)` | none | A status icon in the menu bar ([13. App](13-app.md)) |
| `.fullscreen_bar(FullscreenBar)` | `Solid` | How the title bar is drawn over the content in full screen ([9](09-titlebars.md#the-bar-in-full-screen-fullscreenbar)) |
| `.fullscreen_push(bool)` | `false` | In full screen, the bar pushes the content down instead of covering it ([9](09-titlebars.md#pushing-the-content-fullscreen_push)) |
| `.hud(bool)` | `false` | The HUD panel look ([Utility windows](#utility-windows)) |
| `.panel(PanelBehavior)` | `PanelBehavior::UTILITY` | How a utility window behaves; ignored by main windows |
| `.theme(Theme)` | `Theme::tengen()` | The [`Theme`](../../agents/styles.md#levels) of every control of the window, its title bar and full-screen bar included. `Application::theme` sets it |

`FullscreenBar` has three variants, `Solid` (default), `Glass` and
`ProgressiveGlass`; their look is in
[9. Titlebars](09-titlebars.md#the-bar-in-full-screen-fullscreenbar).

The size and the maximized state are saved on close under `<app_id>.window`
(see restoration in [13. App](13-app.md)). A window closed in full screen
saves nothing; a maximized one keeps its last normal size and reopens
maximized.

## Window controls

`WindowControls::get()` returns the controls of the window the component lives
in.

| Command | Effect |
|---|---|
| `close()` | Closes the window. The process ends when the last main window closes |
| `minimize()` | Minimizes it |
| `toggle_maximize()` | Maximizes or restores it |
| `toggle_fullscreen()` | Enters or leaves full screen |
| `move_window()` | Starts a compositor move (`xdg_toplevel.move`) with the last pointer press |
| `resize(WindowEdge)` | Starts a compositor resize from that edge with the last pointer press |
| `set_title(title)` | Changes the title the compositor and the menu bar show |
| `set_bar(Bar)` | Tells the compositor which bar the window draws ([9](09-titlebars.md#corner-radius)) |
| `set_background_alpha(alpha)` | Multiplies the alpha of the window background, clamped to 0…1 (1 is opaque) |
| `present(token)` | Activates the window with an xdg-activation token, for example one received from a second launch |
| `open_window(app, options)` | Opens another main window in the same process |
| `set_ime_cursor_area(x, y, width, height)` | Tells the input method where the text cursor is, in logical pixels |
| `copy_rich(text, html)`, `primary_selection()` | Clipboard ([13. App](13-app.md)) |

| Query | Returns |
|---|---|
| `status` | `State<WindowStatus>` ([Window state](#window-state)) |
| `title`, `fixed` | The initial title; whether the window is fixed-size |
| `hud`, `panel` | Whether it is a HUD window; whether it is a utility window |
| `ime_preedit` | `State<String>` with the input method's composition text, empty when there is none. Committed text arrives as `Key::Character` events |
| `fullscreen_bar`, `fullscreen_push` | `State`s with the current full-screen bar options. Writing them changes the bar live |
| `focus_bar`, `focus_bar_progress`, `focus_bar_height`, `focus_overlay` | Malaysia's focus mode: whether the compositor bar is down, its animation progress (0…1), its height, and whether the window draws its title bar over the content ([9](09-titlebars.md#maximized-full-screen-and-focus-mode)) |
| `traffic_lights_shown()` | `false` in full screen while the compositor bar is up |
| `titlebar_overlaid()` | `true` in full screen when the title bar is drawn over the content |
| `titlebar_overlay_top()` | The top of the overlaid title bar while it shows |
| `fullscreen_content_top()` | The space to leave at the top of content that sits under the overlaid title bar ([9](09-titlebars.md#what-the-app-does)) |

## Window state

`WindowStatus` arrives with every compositor configure. Read it with
`WindowControls::get().status.read()`:

| Field | Meaning |
|---|---|
| `activated` | The window has focus. `true` before the first configure |
| `maximized` | The compositor maximized it |
| `fullscreen` | It is in full screen (Malaysia's focus mode included) |
| `decoration` | `DecorationResponse::Client` (the app draws the title bar) or `Server` (the compositor does; `Titlebar` then takes no space) |
| `fixed` | Minimum size equals maximum size |

The look of the title bar in each state is in
[9. Titlebars](09-titlebars.md#maximized-full-screen-and-focus-mode).

### Resizing from the edges

With `Client` decoration, the runner reserves a band inside the window:
`WINDOW_RESIZE.edge` (5 px) on each side and `WINDOW_RESIZE.corner` (16 px)
along each corner.

1. Over the band, the cursor shows the arrow for that edge or corner and the
   app stops receiving the motion.
2. A left-button press starts a compositor resize (`xdg_toplevel.resize`).
3. There is no band when the window is maximized, in full screen, fixed-size
   or server-decorated, nor while a button is held.

`WindowEdge` names the edge: `Top`, `Bottom`, `Left`, `Right`, `TopLeft`,
`TopRight`, `BottomLeft`, `BottomRight`. An app that draws its own grip calls
`WindowControls::get().resize(edge)` from a press.

## Utility windows

A utility window is a floating panel, like SwiftUI's `UtilityWindow` or an
inspector. `app::UtilityWindow` is in the prelude:

```rust
UtilityWindow::new(tr!("inspector"), "notes", || Inspector)
    .options(|window| window.size((260, 360)))
    .open();
```

`UtilityWindow::new(title, name, content)` builds the `WindowOptions` from
`title` and `name`, as `WindowOptions::new` does. `content` is a closure that
returns the view. `.options(|window| ..)` changes those options. Unlike
`Application::window` ([13](13-app.md)), it adds no title bar and no
background: `content` draws its own `Titlebar` if it wants one.

1. Call `.open()` from a component of the window that owns the panel.
2. The runner creates another `xdg_toplevel` with `set_parent` towards that
   window, so the compositor keeps it on top and groups it with it.
3. It takes the parent's `app_id` and neither saves nor restores its size.
4. Before its first commit it declares itself a panel
   (`infinity_window_style_v1.set_panel`, version 3) with
   `WindowOptions::panel`. A panel is never the main window: while it holds
   the keyboard, its parent stays active. Without version 3 it behaves like a
   normal window.
5. A `Titlebar` inside it switches to `close_only`: a panel does not minimize.
6. Esc closes it when no text field is focused. Menu shortcuts pressed in it
   go to its parent's menu, as in an `NSPanel`.
7. Closing it removes only the panel; the app keeps running.

`PanelBehavior` has three flags; `PanelBehavior::UTILITY` sets all three, as
SwiftUI's `UtilityWindow` does:

| Field | Like | Effect |
|---|---|---|
| `hides_on_deactivate` | [`NSWindow.hidesOnDeactivate`](https://developer.apple.com/documentation/appkit/nswindow/hidesondeactivate) | The compositor hides it while another app is active |
| `floating` | [`NSPanel.isFloatingPanel`](https://developer.apple.com/documentation/appkit/nspanel/isfloatingpanel) | It stays above every window of its app |
| `key_only_if_needed` | [`NSPanel.becomesKeyOnlyIfNeeded`](https://developer.apple.com/documentation/appkit/nspanel/becomeskeyonlyifneeded) | A press does not give it the keyboard. The kit asks for it, with an [xdg-activation](https://wayland.app/protocols/xdg-activation-v1) token from that press, only when the press lands on a text field or starts a title bar drag |

`WindowOptions::hud(true)` gives a window the HUD look of `NSPanel`
(`hudWindow`), which the HIG keeps for photo, video and slide apps
([Panels](https://developer.apple.com/design/human-interface-guidelines/panels)):

1. The window clears with `HUD_BACKGROUND`, which is translucent, so it
   declares no opaque region and the compositor blurs what is behind it.
   Apple does not publish the material color; `#1e1e1e` at 85 % is an
   approximation.
2. All its content uses the dark scheme, whatever the app's is.
3. Its `Titlebar` switches to `close_only`.
4. The content should not paint its own background, so the material shows.

## Popups

A popup is an `xdg_popup` attached to the window: menus, pop-up buttons,
popovers and tooltips use one ([7. Menus and popovers](07-menus-and-popovers.md)).
For your own:

```rust
use bk::platform::{PopupAnchor, PopupGravity, PopupSpec, Popups};

let spec = PopupSpec {
    anchor_rect: (x, y, width, height),
    anchor: PopupAnchor::Bottom,
    gravity: PopupGravity::Bottom,
    size: (200, 120),
    grab: true,
    ..PopupSpec::default()
};
let id = Popups::get().open(spec, AppComponent::new(MyPopup));
// later
Popups::get().close(id);
```

`PopupSpec` fields map to `xdg_positioner`:

| Field | Default | What it is |
|---|---|---|
| `anchor_rect` | `(0, 0, 0, 0)` | `(x, y, width, height)` of the anchor rectangle, in the parent's logical pixels |
| `anchor` | `TopLeft` | The point of the anchor rectangle the popup attaches to (`PopupAnchor`) |
| `gravity` | `TopLeft` | The direction the popup grows from that point (`PopupGravity`) |
| `offset` | `(0, 0)` | A shift from the attachment point |
| `size` | `(100, 100)` | The popup size in logical pixels |
| `constraint_adjustment` | all off | What the compositor may do when the popup does not fit |
| `grab` | `false` | Takes the pointer and keyboard grab. A click outside then dismisses it |
| `on_dismissed` | `None` | Called when the popup closes, by `close` or by the compositor |

`PopupAnchor` and `PopupGravity` have the same eight values: `TopLeft`,
`Top`, `TopRight`, `Left`, `Right`, `BottomLeft`, `Bottom`, `BottomRight`.

`PopupConstraintAdjustment` has five flags: `flip_x`, `flip_y` (mirror to the
other side), `slide_x`, `slide_y` (move along the edge) and `resize_y` (shrink
the height).

Rules the runner applies:

1. `Popups::get().open` called from inside a popup opens a child of that
   popup; from a window, a child of the window.
2. A popup can only grab if its parent popup grabbed too (`xdg_popup.grab`).
3. Closing a popup closes its children first. Closing a window closes its
   popups.
4. `open` returns a `PopupId` at once; the surface is created on the next
   turn of the event loop.

Inside the popup's content, `PopupHandle::get()` gives the popup itself:

| Method | Effect |
|---|---|
| `id()` | Its `PopupId` |
| `close()` | Closes it |
| `reposition(spec)` | Moves it with a new `PopupSpec` (`xdg_popup.reposition`) |
| `open_child(spec, content)` | Opens a child popup |
| `opens_left()` | `true` when the compositor placed it to the left of its anchor (a flipped submenu) |
| `scroll_shift()`, `set_scroll_shift(f32)` | How far its content is scrolled, so children anchor to the visible row |

## Timers

`Timers::get()` runs code on the UI thread later. The runner polls and redraws
after each callback.

| Method | Effect |
|---|---|
| `after(delay, f)` | Calls `f` once after `delay` |
| `wakeup(f) -> WakeHandle` | Registers `f` and returns a handle |

`WakeHandle` is `Send`: another thread calls `wake()` to make the UI thread
run every registered `wakeup` callback. Use it to show the result of
background work:

```rust
let wake = Timers::get().wakeup(move || result.set(take_result()));
std::thread::spawn(move || {
    compute();
    wake.wake();
});
```

A repeating tick is an `after` that schedules the next one
([recipe 9](../../agents/recipes.md#9-a-stopwatch)).

## Input helpers

`inserts_text(modifiers)` is `true` when a key press with those modifiers
types text: neither Control, Alt nor Super is held. Shift alone still types.

## Menu address

`MainMenuAddress { service, path }` is the D-Bus address of a window's menu
bar menu. The runner publishes it to the compositor with
`org_kde_kwin_appmenu`; a shell reads it back for each window
([18. Platform](18-platform.md)). `ObjectId` is the Wayland object id
(re-exported from `wayland-backend`) that identifies a surface in those
lists.

## Errors

`run_window` returns `RunError` when the window cannot run:

| Variant | Raised when |
|---|---|
| `NoDisplay` | `$WAYLAND_DISPLAY` names no reachable socket |
| `Wayland`, `Registry` | The connection or the registry fails |
| `MissingGlobal(name)` | A [needed global](18-platform.md#wayland-protocols) is missing, `zwlr_layer_shell_v1` for `run_layer`, or `ext_session_lock_manager_v1` for `run_lock` |
| `Vulkan(reason)` | No `libvulkan`, no GPU that can present, or the surface lacks the swapchain format |
| `Skia` | Skia cannot create its Vulkan context |
| `DeviceLost` | The GPU device cannot be recreated, or is lost again after two recreations without a presented frame |
| `SurfaceLost` | Never returned: the runner creates a new Vulkan surface and swapchain |
| `LayerSize` | A `LayerSpec` leaves a size at 0 without anchoring both opposite edges |
| `EventLoop(reason)` | A `calloop` source cannot be created or the loop fails |
| `Io` | The appearance watcher cannot start |

Next: [9. Titlebars](09-titlebars.md).
