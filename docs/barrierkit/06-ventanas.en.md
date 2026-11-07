# 6. Windows

**Goal:** open windows with the right options and control their size, state
and popups.
**Reader:** anyone who defines the main window of an app.
**See also:** the look of the titlebar and the traffic lights is in chapter
[7](07-titlebars.en.md).

## Opening a window

```rust
run_window(
    AppComponent::new(MiApp),
    WindowOptions::new("Notas", "notas")
        .size((760, 520))
        .min_size(Some(WINDOW_MIN_SIZE))
        .menu(menu),
)
```

| Option | Effect |
|---|---|
| `new(título, nombre)` | Visible title and short name. The `app_id` is `io.github.hblanqueto.malaysia.<nombre>` unless you pass `.app_id(..)` |
| `.size((ancho, alto))` | Initial size (640 × 420 by default). If a saved one exists, it is restored |
| `.min_size(..)`, `.max_size(..)` | Limits. If they are equal, the window has a fixed size |
| `.menu(AppMenu)` | Global menu of the window (chapter [11](11-app.en.md)) |
| `.fullscreen_bar(FullscreenBar)` | How the overlaid titlebar is drawn in full screen; `Solid` by default (chapter [7](07-titlebars.en.md#the-bar-in-full-screen-fullscreenbar)) |
| `.fullscreen_push(bool)` | In full screen, the bar pushes all the content instead of covering it; off by default (chapter [7](07-titlebars.en.md#pushing-the-content-fullscreen_push)) |

Inside the app, `WindowControls::get()` gives the state (`activated`,
`maximized`, `fullscreen`, `decoration`, `fixed`) and the commands: `close()`,
`minimize()`, `toggle_maximize()`, `toggle_fullscreen()`, `move_window()`,
`resize(WindowEdge)`, `set_title(title)`, which changes the title the
compositor and the menu bar see, `set_background_alpha(alpha)`, which
multiplies the alpha of the background the runner clears the window with (1 is
opaque), and `open_window(app, options)`, which opens another window in the
same process; the process ends when the last one closes.

With `zwp_text_input_v3`, the input method's composition text reaches
`WindowControls::get().ime_preedit` (a `State<String>`, empty when there is
none) and the committed text arrives as a `Key::Character` key.
`set_ime_cursor_area(x, y, width, height)` tells the input method where the
cursor is, in the window's logical pixels.

`primary_selection()` gives a `ClipboardProvider` for the primary selection
(`zwp_primary_selection`), the one the middle button pastes: `set_contents`
publishes it and `get_contents` reads another app's. `copy_rich(text, html)`
copies the plain text to the clipboard and also `text/html`, so apps that
understand it paste with formatting.

## Utility windows

```rust
open_utility_window(AppComponent::new(Inspector), WindowOptions::new("Inspector", "Vista").size((260, 360)));
```

An app opens floating panels, like SwiftUI's `UtilityWindow` or the Preview
inspector, with `barrier_kit::platform::open_utility_window`:

1. Call it from a component of the window that opens it.
2. The runtime creates another `xdg_toplevel` with `set_parent` towards that
   window, so the compositor keeps it on top and groups it with it.
3. It uses the parent window's `app_id` and neither saves nor restores its
   size, so it does not overwrite the main one's.
4. Closing it removes only it; the app goes on while another window remains.
5. Before its first commit it declares itself a panel (`set_panel`, version 3
   of `infinity_window_style_v1`) with its `WindowOptions::panel`. By default,
   `PanelBehavior::UTILITY`, the same as SwiftUI's `UtilityWindow`: it hides
   while the app is not active
   ([`hidesOnDeactivate`](https://developer.apple.com/documentation/appkit/nswindow/hidesondeactivate)),
   floats above its app's windows and only takes the keyboard when needed
   ([`becomesKeyOnlyIfNeeded`](https://developer.apple.com/documentation/appkit/nspanel/becomeskeyonlyifneeded)).
   A panel is never the main window: while it holds the keyboard, the window
   it came from stays active. The compositor applies it; without version 3
   the window behaves like a normal one.

`WindowOptions::hud(true)` gives it the HUD style of `NSPanel`
(`hudWindow`): a panel that is "darker and translucent", which the HIG keeps
for photo, video or slide apps
([Panels](https://developer.apple.com/design/human-interface-guidelines/panels)):

1. The window clears with `HUD_BACKGROUND`, which is translucent, so it
   declares no opaque region and the compositor blurs what is behind it.
2. All its content uses the dark scheme (`use_appearance`), whatever the
   app's is.
3. Its `Titlebar` switches to `close_only`: a thin bar, a transparent
   background and only close enabled, because the HIG asks not to offer
   minimize in a panel.
4. The content should not paint its own background so the material shows.
   Apple does not publish the material color; `#1e1e1e` at 85 % is an
   approximation.

## Menu bar icon

```rust
let icono = StatusItem::new("correo", "Correo", "mail-unread")
    .items(vec![MenuItem::action("nuevo", "Nuevo mensaje")]);
run_window(app, WindowOptions::new("Correo", "Correo").status_item(icono))?;
use_menu_action(|id| if id == STATUS_ITEM_ACTIVATE { /* mostrar la ventana */ });
```

`WindowOptions::status_item` publishes a status icon, AppKit's
`NSStatusItem`, over the
[StatusNotifierItem](https://specifications.freedesktop.org/status-notifier-item/latest/) protocol:

1. The kit exports `org.kde.StatusNotifierItem` at `/StatusNotifierItem`,
   named `org.kde.StatusNotifierItem-<pid>-<n>`, with the title, an icon
   theme `IconName` and its menu as dbusmenu.
2. It registers with `org.kde.StatusNotifierWatcher`. If the watcher shows up
   later (the shell starts later or restarts), it registers again.
3. Each menu item reaches `use_menu_action` with its `id`, like the app menu.
   `Activate` arrives as `STATUS_ITEM_ACTIVATE`.
4. The menu and the object share one connection, because the specification
   looks up `Menu` on the icon's own bus name.

The specification spells the interfaces `org.freedesktop.*`, but KDE,
libayatana-appindicator and waybar use `org.kde.*`, and so does the kit.

## Window state

The state arrives with each compositor configure and is read with
`WindowControls::get().status.read()`: `activated`, `maximized`, `fullscreen`,
`decoration` and `fixed`. How the titlebar looks in each state is in chapter
[7](07-titlebars.en.md#maximized-full-screen-and-focus-mode).

## Resizing from the edges

With `Client` decoration, the runner reserves a band inside the window of
`WINDOW_RESIZE.edge` (5 px) on each side and `WINDOW_RESIZE.corner` (16 px)
along each corner:

1. Over the band, the cursor turns into the arrow for that edge or corner and
   the app stops receiving the motion.
2. A left-button press starts a compositor resize (`xdg_toplevel.resize`).
3. There is no band when the window is maximized, in full screen, fixed-size or
   server-decorated, nor while the app is dragging something.

## The green button and double click

The user decides them in `window.domain`. With a fixed-size window (minimum
equal to maximum) they change:

| `green_button` | Normal window | Fixed size |
|---|---|---|
| `focus` (default) | Full screen | Maximizes |
| `zoom` | Maximizes | Maximizes |
| `fullscreen` | Full screen | Maximizes |
| `none` | Nothing; the button is drawn gray and without a glyph | Nothing; the button is drawn gray and without a glyph |

| `double_click_titlebar` | Normal window | Fixed size |
|---|---|---|
| `zoom` (default), `fill` | Maximizes | Nothing |
| `minimize` | Minimizes | Minimizes |
| `none` | Nothing | Nothing |

Both tables live in `vow_appearance::window`: `read_window_behavior()` reads the
domain and `green_action` and `double_click_action` give the action. The
compositor's chrome uses the same functions.

## Popups

Menus, pop-up buttons and tooltips use `xdg_popup`. For your own:

```rust
let id = Popups::get().open(spec, AppComponent::new(MiPopup));
Popups::get().close(id);
```

`PopupSpec` sets the anchor rectangle, the anchor, the gravity, the size, the
adjustment when it does not fit and whether it takes the pointer grab.

Next: [7. Titlebars](07-titlebars.en.md).
