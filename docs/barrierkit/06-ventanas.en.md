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
        .decorations(Decorations::Client)
        .size((760, 520))
        .min_size(Some(WINDOW_MIN_SIZE))
        .menu(menu),
)
```

| Option | Effect |
|---|---|
| `new(título, nombre)` | Visible title and short name. The `app_id` is `io.github.hblanqueto.malaysia.<nombre>` unless you pass `.app_id(..)` |
| `.decorations(..)` | `Client` (default): the kit draws the titlebar. `Server`: it asks the compositor for it; if there is no `zxdg_decoration_manager_v1`, it falls back to `Client` |
| `.size((ancho, alto))` | Initial size (640 × 420 by default). If a saved one exists, it is restored |
| `.min_size(..)`, `.max_size(..)` | Limits. If they are equal, the window has a fixed size and the green button maximizes it vertically only |
| `.parent(..)` | Child window (dialog) |
| `.menu(AppMenu)` | Global menu of the window (chapter [11](11-app.en.md)) |
| `.fullscreen_bar(FullscreenBar)` | How the overlaid titlebar is drawn in full screen; `Solid` by default (chapter [7](07-titlebars.en.md#the-bar-in-full-screen-fullscreenbar)) |
| `.fullscreen_push(bool)` | In full screen, the bar pushes all the content instead of covering it; off by default (chapter [7](07-titlebars.en.md#pushing-the-content-fullscreen_push)) |

Inside the app, `WindowControls::get()` gives the state (`activated`,
`maximized`, `fullscreen`, `decoration`, `fixed`) and the commands: `close()`,
`minimize()`, `toggle_maximize()`, `toggle_fullscreen()`, `move_window()`,
`resize(WindowEdge)`.

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
| `focus` (default) | Full screen | Maximizes vertically |
| `zoom` | Maximizes | Maximizes vertically |
| `fullscreen` | Full screen | Maximizes vertically |
| `none` | Nothing; the button is drawn gray and without a glyph | Nothing; the button is drawn gray and without a glyph |

| `double_click_titlebar` | Normal window | Fixed size |
|---|---|---|
| `zoom` (default), `fill` | Maximizes | Nothing |
| `minimize` | Minimizes | Minimizes |
| `none` | Nothing | Nothing |

## Popups

Menus, pop-up buttons and tooltips use `xdg_popup`. For your own:

```rust
let id = Popups::get().open(spec, AppComponent::new(MiPopup));
Popups::get().close(id);
```

`PopupSpec` sets the anchor rectangle, the anchor, the gravity, the size, the
adjustment when it does not fit and whether it takes the pointer grab.

Next: [7. Titlebars](07-titlebars.en.md).
