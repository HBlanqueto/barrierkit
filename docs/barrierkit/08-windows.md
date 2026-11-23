# 8. Windows

**Goal:** open a window, set its options, read and change
its state, open utility windows and popups, and run code later.
**Scope:** the window API of `bk::platform`: `run_window`,
`WindowOptions`, `WindowControls`, `WindowStatus`, scenes (`WindowGroup`,
`Window`, `Settings`, `openWindow`, `dismissWindow`), utility windows, popups,
timers, input helpers and `RunError`. `Application` is in
[2. Your first app](02-first-app.md). The title bar and the traffic lights are
in [9. Titlebars](09-titlebars.md). How the runner works inside (Vulkan,
Wayland protocols) is in [18. Platform](18-platform.md).
**Reader:** anyone who builds the window of an app.
**Input:** an app that runs with `Application` ([2](02-first-app.md)).
**Output:** an app that controls its own window.

## Steps

1. Open the window with `Application`
   ([2](02-first-app.md#what-application-does-for-you)). A window that draws
   its own title bar (a sidebar, toolbar items) sets `WindowStyle::HiddenTitleBar`
   or `Plain` and gets no kit `Titlebar` ([ADR 0115](../decisions.md#0115--standard-menus-by-default-edit-items-reach-the-focused-text-field-macos-menus-are-appkits)).
   An app with more than one window declares [scenes](#scenes) on the
   `Application`.
2. Describe the window with `WindowOptions` in `.options(..)`:

   ```rust
   use bk::prelude::*;

   fn main() -> Result<(), RunError> {
       Application::new("notes")
           .window(tr!("notes"), || {
               rect()
                   .expanded()
                   .child(Titlebar::new().title(tr!("notes")))
           })
           .options(|window| {
               window
                   .size((760, 520))
                   .min_size(Some(WINDOW_MIN_SIZE))
                   .window_style(WindowStyle::HiddenTitleBar)
           })
           .run()
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
| `.min_size(Some((w, h)))`, `.max_size(..)` | `None` | Limits of the content below the window's bar, as SwiftUI's `.frame(minWidth:minHeight:)`: the window adds 52 for `HiddenTitleBar`, the native bar for `TitleBar` (40 on Linux) and 0 for `Plain` (ADR 0117). Equal limits make the window fixed-size (`WindowStatus::fixed`); a 0 in the maximum leaves that dimension unbounded |
| `.menu(MainMenu)` | none | The window's menu bar menu ([13. App](13-app.md)) |
| `.status_item(StatusItem)` | none | A status icon in the menu bar ([13. App](13-app.md)) |
| `.fullscreen_bar(FullscreenBar)` | `Solid` | How the title bar is drawn over the content in full screen ([9](09-titlebars.md#the-bar-in-full-screen-fullscreenbar)) |
| `.fullscreen_push(bool)` | `false` | In full screen, the bar pushes the content down instead of covering it ([9](09-titlebars.md#pushing-the-content-fullscreen_push)) |
| `.window_style(WindowStyle)` | `TitleBar` | Who frames the window on macOS ([Window style](#window-style)); Linux ignores it |
| `.opaque(bool)` | `true` | With `false` the window is non-opaque on macOS and its sidebar glass is the system's material ([15](15-backdrops-and-media.md#on-macos)); Linux ignores it |
| `.hud(bool)` | `false` | The HUD panel look ([Utility windows](#utility-windows)) |
| `.panel(PanelBehavior)` | `PanelBehavior::UTILITY` | How a utility window behaves; ignored by main windows |
| `.theme(Theme)` | `Theme::tengen()` | The [`Theme`](../../agents/styles.md#levels) of every control of the window, its title bar and full-screen bar included. `Application::theme` sets it |
| `scene` (field) | `None` | The scene id that names the saved size: `<app_id>.window.<scene>`. `Application` sets it; `None` is `<app_id>.window` |

`FullscreenBar` has three variants, `Solid` (default), `Glass` and
`ProgressiveGlass`; their look is in
[9. Titlebars](09-titlebars.md#the-bar-in-full-screen-fullscreenbar).

The size and the maximized state are saved on close under `<app_id>.window`,
or `<app_id>.window.<scene>` for a scene other than `MAIN_WINDOW`
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
| `make_key_and_order_front()` | Brings the window to the front and gives it the keyboard with the last key or button press, for example from a shortcut in another window of the same process |
| `open_window(app, options)` | Opens another main window in the same process. [Scenes](#scenes) use it |
| `set_ime_cursor_area(x, y, width, height)` | Tells the input method where the text cursor is, in logical pixels |
| `copy_rich(text, html)`, `primary_selection()` | Clipboard ([13. App](13-app.md)) |

| Query | Returns |
|---|---|
| `status` | `State<WindowStatus>` ([Window state](#window-state)) |
| `title`, `fixed`, `fixed_width` | The initial title; whether the window is fixed-size; whether its minimum and maximum widths are equal |
| `hud`, `panel` | Whether it is a HUD window; whether it is a utility window |
| `ime_preedit` | `State<String>` with the input method's composition text, empty when there is none. Committed text arrives as `Key::Character` events |
| `fullscreen_bar`, `fullscreen_push` | `State`s with the current full-screen bar options. Writing them changes the bar live |
| `focus_bar`, `focus_bar_progress`, `focus_bar_height`, `focus_overlay` | Malaysia's focus mode: whether the compositor bar is down, its animation progress (0…1), its height, and whether the window draws its title bar over the content ([9](09-titlebars.md#maximized-full-screen-and-focus-mode)) |
| `desktop` | `State<Option<u32>>`: the number of the Malaysia desktop the window is on, from 1; `None` in a focus space or outside Malaysia (`infinity_focus_v1` 4) |
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
| `decoration` | `DecorationResponse::Client` (the app draws the title bar and the lights), `Server` (the compositor does; `Titlebar` then takes no space) or `Hybrid` (the system draws the lights and the app the bar) |
| `fixed` | Minimum size equals maximum size |

macOS has no public tiled state: a tiled window only moves and resizes
([window designs](../research/csd-window-designs.md#2-designs-against-states)).

The look of the title bar in each state is in
[9. Titlebars](09-titlebars.md#maximized-full-screen-and-focus-mode).

### Window style

`WindowStyle` (like SwiftUI's `windowStyle`) says who frames a main window on
macOS. The compositor decides on Linux, so there it has no effect. Utility
windows and sheets are always `TitleBar`.

| Variant | `NSWindow` | `WindowStatus::decoration` | Use |
|---|---|---|---|
| `TitleBar` | titled, native lights | `Server` | Apps that look like any Mac app |
| `HiddenTitleBar` | titled with `fullSizeContentView`, transparent title bar, an empty `NSToolbar` whose style follows the `Titlebar`'s `toolbar_style`, so the native lights sit where the kit draws its own: (18, 18) under `Unified`, (11, 12) under `UnifiedCompact`, (9, 9) under `Expanded`, as SwiftUI's `.hiddenTitleBar` | `Hybrid` | The kit draws its `Titlebar` and sidebar beside the native lights and draws none of its own |
| `Plain` | borderless, shadow, corner `CORNER_WINDOW.fitted` | `Client` | The kit draws the `Titlebar` and `TrafficLights`, as on Linux |

In an `Application`, a `HiddenTitleBar` or `Plain` window gets no kit
`Titlebar`: the app draws its own and keeps the scenes and the standard menus
([13](13-app.md#standard-menus)).

A `Plain` window moves with `WindowControls::move_window` (a press on the
`Titlebar`): the view moves it while the button is down, because
`performWindowDragWithEvent` does not begin on macOS 27.2. It resizes from the
edge AppKit gives a resizable window. A double click on the `Titlebar` does
what `double_click_titlebar` says. Only the titled styles have the system's
shadow and the Move & Resize menu on the green light; no public API shows that
menu, so the kit's green light has none.

A `Plain` window enters AppKit's full screen (`.fullScreenPrimary` and
`toggleFullScreen`): a Space of its own and the transition of a titled window,
with square corners. The menu bar and the Dock stay hidden, because with the
menu bar AppKit lays its own title strip and dimmed lights over the window's
top edge. The kit hides its lights in full screen, so the app offers a way out
that calls `toggle_fullscreen()`.

```rust
WindowOptions::new("Notes", "notes").window_style(WindowStyle::Plain)
```

The program is [`recipes/notes_sidebar.rs`](../../agents/recipes/notes_sidebar.rs).

In full screen a `HiddenTitleBar` window keeps its empty `NSToolbar`, icons
only as in SwiftUI, and makes AppKit's full-screen bar clear. The kit's
`Titlebar` row and the sidebar keep their windowed place. When the pointer
reaches the top edge, AppKit brings the menu bar down with a 52 pt bar that
holds the lights. While the menu bar is shown, the window moves the glass
controls of the title bar row into that bar, in an
[`NSTitlebarAccessoryViewController`](https://developer.apple.com/documentation/appkit/nstitlebaraccessoryviewcontroller).
They ride in one row beside the lights, as in SwiftUI, and still take clicks;
a popover opened from them follows the row, and with the sidebar hidden the row
makes room for the lights. Under `WindowToolbarStyle::Expanded` the toolbar row
rides instead, in a row of its own under the lights, as SwiftUI's `.expanded`
does. Under the bar, a `Sidebar` gets a soft blurred edge as deep as the bar
reaches, as SwiftUI's sidebar does; the content beside it is left as it is.
The content does not move
([ADR 0116](../decisions.md#0116--in-full-screen-the-title-bar-row-of-a-hiddentitlebar-window-rides-with-appkits-bar)).

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

## Scenes

An `Application` declares its windows as scenes. `.window(title, content)` is
the first, `.scene(..)` adds the others, and `.options(..)` changes the
`WindowOptions` of the scene declared last. The first scene that is not
`Settings` opens at launch; the rest open when the app asks.

| Type | Like | What it is |
|---|---|---|
| `WindowGroup::new(id, title, content)` | `WindowGroup(id:)` | Any number of windows with the same content. `.window(..)` declares one with the id `MAIN_WINDOW` |
| `WindowGroup::for_value(id, title, \|value: T\| view)` | `WindowGroup(id:for:)` | One window per value; `T` is `Clone + PartialEq + 'static` |
| `Window::new(id, title, content)` | `Window(id:)` | One window with that id |
| `Settings::new(content)` | `Settings` | One window with the id `SETTINGS_WINDOW`. `content` draws its own title bar, as [`SettingsWindow`](12-patterns.md#settingswindow) does |

`Scene` is what `.scene(..)` takes: each type above converts into it. A
`WindowGroup` or `Window` window gets the same background and `Titlebar` as
`Application::window`.

`Application` applies its name and its `Theme` when `.scene(..)` takes a scene,
so call `Application::theme` first. To give one scene another theme or size,
call `Application::options` right after its `.scene(..)`.

The program is [`recipes/scenes.rs`](../../agents/recipes/scenes.rs).

Open and close from a component of a scene window
([recipe 25](../../agents/recipes.md#25-several-windows)):

| Hook | Returns | Methods |
|---|---|---|
| `use_open_window()` | `OpenWindowAction` | `open(id)`; `open_value(value)` finds the group by the type of `value` |
| `use_dismiss_window()` | `DismissWindowAction` | `dismiss()` closes this window; `dismiss_id(id)` closes every open window with that id |

Both panic where there is no `Application` above (a test, a `run_window` app).
A value in `open_value` or an id that no scene declares logs a warning and does
nothing.

Rules the runner applies:

1. `open(id)` on a `Window` or `Settings` fronts the window if it is open and
   opens it otherwise. On a `WindowGroup` it opens a new window each time. On a
   value group, `open_value` fronts the window whose value is equal, or opens
   a new one.
2. Opening sends `WindowControls::open_window` from the calling window. The
   runner makes another `xdg_toplevel` with smithay-client-toolkit
   (`Runtime::create_window`, as `run_window` does for the first window), so
   `freya-winit` is not involved ([ADR 0033](../decisions.md)).
3. Every scene window shares the app's `app_id`.
4. Wayland has no list of windows and no way to raise another window. The kit
   keeps its own list: a scene window adds its `WindowControls` when it mounts
   and removes it when it closes. Fronting is `make_key_and_order_front` on the
   listed controls, which asks the compositor with the last input serial.
   The [Window menu](13-app.md#the-window-menu) shows that list.
5. Each scene saves its own size (`WindowOptions::scene`).
6. The app is not restored with the windows that were open, and no scene
   sets its position: xdg-shell gives a client neither.

Not built: `defaultLaunchBehavior`, `openSettings` and `SettingsLink`,
`openWindow(id:value:)`, `dismissWindow(value:)`, `handlesExternalEvents`, a
`Codable` value that restores the window, and a Window menu title that follows
`set_title` (the list keeps the title the scene declared).

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
5. `open_popover(spec, content, shape)` opens a popup with a popover's
   corner and arrow ([7. Menus and popovers](07-menus-and-popovers.md#other-popover-shapes)).
   `reposition(id, spec)` moves an open popup with a new `PopupSpec`.

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
| `every(interval, f)` | Calls `f` every `interval`, for as long as the app runs |
| `every_minute(f)` | Calls `f` at the start of each minute of the clock, the first time within the minute |
| `wakeup(f) -> WakeHandle` | Registers `f` for as long as the app runs and returns a handle |

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

To draw again at a steady pace while something runs, call `use_timeline(active, every)`
from `bk::app` on every render, like SwiftUI's `TimelineView(.periodic(from:by:))`. It
renders the component every `every` while `active` is `true` and stops when it is `false`
([recipe 9](../../agents/recipes.md#9-a-stopwatch)).

```rust
// ✗ an `after` that schedules the next `after`
// ✓
use_timeline(running, Duration::from_millis(10));
```

## Input helpers

`inserts_text(modifiers)` is `true` when a key press with those modifiers
types text: neither Control, Alt nor Super is held. Shift alone still types.

`modifier_flags()` returns the modifier keys held now, for a pointer handler:
a pointer event carries none.

## Menu address

`MainMenuAddress { service, path }` is the D-Bus address of a window's menu
bar menu. The runner publishes it to the compositor with
`org_kde_kwin_appmenu`; a shell reads it back for each window
([18. Platform](18-platform.md)). `ObjectId` is the Wayland object id
(re-exported from `wayland-backend`) that identifies a surface in those
lists.

## Documents and file dialogs

SwiftUI: `.fileImporter`, `.fileExporter`, `.fileDialogDefaultDirectory`, `.onOpenURL`
and `DocumentGroup`; AppKit: `NSOpenPanel`, `NSSavePanel` and `NSDocument`. The programs are
[`recipes/file_dialogs.rs`](../../agents/recipes/file_dialogs.rs) and
[`recipes/documents.rs`](../../agents/recipes/documents.rs).

1. **Choose files.** `.file_importer(is_presented, types, multiple, on_completion)` and `.file_exporter(is_presented, "Name.png", types, on_completion)` take a `State<bool>`, as `sheet` does. `types` is a `Vec<ContentType>`: `ContentType::extension("png")` or `ContentType::Folder`. An empty list allows every file. `on_completion` runs only when the user chose.
2. **Start folder.** `.file_dialog_default_directory(path)` wraps a view; the dialogs of the views below it start there. For a dialog of your own use `FileDialog::open()` or `FileDialog::save(name)`, `.directory(path)` and `.present(&WindowControls::get()).await`.
3. **Where it opens.** On macOS the dialog is a sheet of the window. On Linux the portal opens it without a parent, filters or start folder.
4. **Make a window a document.** Call `use_document(|path| write(path))` in the window's component. It returns a `Document`.
5. **Keep it dirty.** `document.set_edited(true)` when the content differs from the file, `false` when it matches. An edited window shows the dot in its close button.
6. **Save.** `document.save().await` writes through your closure, asking for a file first when `document.url()` is `None`. Use it for the Save command.
7. **Close.** The native close button of an edited window shows the "Do you want to save the changes made to the document “name”?" sheet with Save, Cancel and Don’t Save. Save writes and closes; a write that fails keeps the window open. Cmd-W and Cmd-Q go through the same delegate methods and are meant to ask the same, but were not run.
8. **Open.** `.on_open_url(|url: OpenedUrl| ..)` runs for each file (`OpenedUrl::File(path)`) or address (`OpenedUrl::Other(text)`) that Finder, `open` or a link hands to the app. The app declares `CFBundleDocumentTypes` and `CFBundleURLTypes` in its `Info.plist`; the kit does not generate it.

`WindowControls::close()` and the kit's own traffic lights (a `Plain` window) close without asking. What is not built is in [ADR 0112](../decisions.md).

## Errors

`run_window` returns `RunError` when the window cannot run:

| Variant | Raised when |
|---|---|
| `NoDisplay` | `$WAYLAND_DISPLAY` names no reachable socket |
| `Wayland`, `Registry` | The connection or the registry fails |
| `MissingGlobal(name)` | A [needed global](18-platform.md#wayland-protocols) is missing, `zwlr_layer_shell_v1` for `run_layer`, or `ext_session_lock_manager_v1` for `run_lock` |
| `Vulkan(reason)` | No `libvulkan`, no GPU that can present, or the surface lacks the swapchain format |
| `Skia` | Skia cannot create its Vulkan context |
| `DeviceLost` | The GPU device cannot be recreated, or is lost again after two recreations without a presented frame; a swapchain image not ready within one second counts as a loss |
| `SurfaceLost` | Never returned: the runner creates a new Vulkan surface and swapchain |
| `LayerSize` | A `LayerSpec` leaves a size at 0 without anchoring both opposite edges |
| `NoWindow` | `Application::run` finds no scene to open at launch (only `Settings`, or none) |
| `EventLoop(reason)` | A `calloop` source cannot be created or the loop fails |
| `Io` | The appearance watcher cannot start |

Next: [9. Titlebars](09-titlebars.md).
