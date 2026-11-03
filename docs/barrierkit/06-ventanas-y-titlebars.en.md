# 6. Windows and titlebars

**Goal:** open windows with the right options and choose the titlebar and
traffic light type.
**Reader:** anyone who defines the main window of an app.

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
| `.menu(AppMenu)` | Global menu of the window (chapter [10](10-app.en.md)) |

Inside the app, `WindowControls::get()` gives the state (`activated`,
`maximized`, `fullscreen`, `decoration`, `fixed`) and the commands: `close()`,
`minimize()`, `toggle_maximize()`, `toggle_fullscreen()`, `move_window()`,
`resize(WindowEdge)`.

## The horizontal titlebar: `Titlebar`

```rust
Titlebar::new()
    .title("Documento")
    .with_sidebar(true)
    .left(SidebarToggle)
    .right(IconButton::new(Symbol::Plus, "Nuevo"))
```

- It has three areas: `left`, the centered title and `right`.
- Without a sidebar, the areas are symmetric so the title stays centered.
- With `.with_sidebar(true)` it aligns with the sidebar. The traffic lights
  stay fixed in place even when the sidebar collapses.
- Dragging it moves the window; double-click does what `window.domain` says.
- When `Decorations::Server` is granted, it measures 0 and draws nothing.
- `.close_only(true)` leaves only the close button, for dialogs.

The «Ventana y herramientas» ("Window and tools") pane of Tengen App Show
shows standalone traffic lights, a `Titlebar` with a title and another one with
`.close_only(true)`:

![Traffic lights, title bar and "close only"](assets/apps/tengen-ventana-light.png)

## The traffic lights: `TrafficLights`

Close, minimize and maximize. They are not usually created by hand:
`Titlebar` and `Sidebar::traffic_lights(true)` place them with
`titlebar::window_traffic_lights(enfocada, fija)`. If you need them standalone:

```rust
TrafficLights::new()
    .focused(true)
    .zoom_supported(true)
    .vertical(false)
    .round(false)
    .on_action(|boton: ButtonKind| { /* Close, Minimize, Maximize */ })
```

| Setting | Where | Values |
|---|---|---|
| Color | `appearance.domain` → `traffic_lights` | `auto`, `standard` (or `macos`), `accent`, `wallpaper` |
| Green button | `window.domain` → `green_button` | `focus` (default: full screen), `zoom`, `fullscreen`, `none` |
| Double-click on the titlebar | `window.domain` → `double_click_titlebar` | `zoom` (default), `fill`, `minimize`, `none` |

In an inactive window the traffic lights turn gray.

## Five titlebar types: `WindowChrome`

`chrome::WindowChrome::new(estilo, contenido)` wraps the window and places the
traffic lights and the title according to a `ChromeStyle`:

```rust
let estilo = ChromeStyle {
    layout: TitlebarLayout::Rail,
    side: Side::Left,
    rail_width: TITLEBAR_RAIL.width,
    round_lights: false,
    reveal: Reveal::Always,
    resting_edge: RestingEdge::None,
};
WindowChrome::new(estilo, SplitView::new(barra, detalle)).footer(pie)
```

| `TitlebarLayout` | What it is | Screenshot (Rótulo) |
|---|---|---|
| `Top` (default) | Horizontal titlebar at the top | ![Top layout](assets/apps/rotulo-top.png) |
| `Rail` | Vertical strip on one side with the traffic lights in a column and the title rotated 90° | ![Rail layout](assets/apps/rotulo-rail.png) |
| `Unified` | The traffic lights in a column inside the sidebar, with no strip of their own. Without a visible sidebar it becomes `Rail` | ![Unified layout](assets/apps/rotulo-unified.png) |
| `Pill` | A floating pill with the traffic lights. In full screen it is glass | ![Pill layout](assets/apps/rotulo-pill.png) |
| `Actions` | A wider `Rail` with an actions footer (`.footer(..)`) | ![Actions layout](assets/apps/rotulo-actions.png) |

| Field | Values | Effect |
|---|---|---|
| `side` | `Left`, `Right` | Side of the rail |
| `rail_width` | `TITLEBAR_RAIL.width` or `.width_wide` | Width of the rail |
| `round_lights` | `bool` | Round or square (default) traffic lights |
| `reveal` | `Always`, `Drawer`, `Push`, `Corner`, `Popup` | Always visible, or it appears when the pointer comes close: as a drawer on top, pushing the content, only in the corner or as a pop-up pill |
| `resting_edge` | `None`, `Line`, `Accent` | What stays visible while it is hidden: nothing, a line or an accent line |

Some combinations make no sense and correct themselves
(`ChromeStyle::effective`): `Unified` with `Drawer`, `Push` or `Popup` becomes
`Corner`; `Pill` with `Drawer` or `Push` becomes `Popup`; in full screen,
`reveal` is always `Always`.

Rail on the right, round traffic lights, flat sidebar and dark mode:

![Right rail with round traffic lights](assets/apps/rotulo-rail-flat-round.png)

The Rótulo app (chapter [14](14-apps-de-prueba.en.md)) tests every combination
live.

## Maximized, full screen and focus mode

The state arrives with each compositor configure and is read with
`WindowControls::get().status.read()`: `maximized`, `fullscreen` and `activated`.

| State | How it is entered | What the kit does |
|---|---|---|
| Normal | — | Titlebar in place; when the size changes, it is saved |
| Maximized | Green button with `green_button = "zoom"`, double click with `zoom` or `fill`, or `toggle_maximize()` | Nothing visual: the titlebar stays the same. In MalaysiaDreams the compositor keeps the frame (corners and shadow). The size is not saved |
| Full screen and focus mode | Green button with `focus` (default) or `fullscreen`, or `toggle_fullscreen()` | The titlebar leaves the layout and the content fills the whole window. The traffic lights hide. The size is not saved |

In MalaysiaDreams, focus mode gives the window the `fullscreen` state and a
black bar that slides down when the pointer nears the top
([MalaysiaDreams ADRs 0051 and 0054](https://github.com/HBlanqueto/malaysia/tree/main/docs/decisiones)).
The kit requests `set_overlay` through `infinity_focus_v1` v2. While the bar is
down, `titlebar_overlaid()` is true and each titlebar is drawn over the
content, at the height of the bar, with progressive glass, appearing with it:

| Layout | In full screen, with the bar down |
|---|---|
| `Top` | The `Titlebar` slides down over the content. The sidebar traffic lights move down by the same amount |
| `Rail`, `Actions` | The rail is drawn as glass and moves down with the bar |
| `Unified` | Same as `Top` |
| `Pill` | The pill is hidden until the bar comes down; then it appears as glass |

In full screen, `reveal` is always `Always`. Without the protocol (another
compositor), the titlebar stays in the layout and only the traffic lights hide.

### What the app does

1. If it uses `Titlebar`, `WindowChrome`, `Sidebar` or `PreferencesWindow`, it
   does nothing: all three states already work.
2. It leaves room for the bar in the content under the overlaid titlebar, as
   Ondina does:

   ```rust
   let arriba = try_consume_root_context::<WindowControls>()
       .map_or(0.0, |controls| controls.fullscreen_content_top());
   rect().padding(Gaps::new(arriba, 0.0, 0.0, 0.0)).child(contenido)
   ```

3. For its own behavior (an immersive mode that hides panels), it reads the
   state and decides:

   ```rust
   let controls = WindowControls::get();
   let inmersivo = controls.status.read().fullscreen;
   rect().maybe_child((!inmersivo).then(panel_de_herramientas))
   ```

4. To offer it in a menu, it calls `toggle_maximize()` or
   `toggle_fullscreen()` from an `Action` (chapter [10](10-app.en.md)).

### The green button and double click

The user decides them in `window.domain`. With a fixed-size window (minimum
equal to maximum) they change:

| `green_button` | Normal window | Fixed size |
|---|---|---|
| `focus` (default) | Full screen | Maximizes vertically |
| `zoom` | Maximizes | Maximizes vertically, even though the button is drawn gray and without a glyph |
| `fullscreen` | Full screen | Maximizes vertically |
| `none` | Nothing | Maximizes vertically |

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

Next: [7. Sidebar and SplitView](07-barra-lateral.en.md).
