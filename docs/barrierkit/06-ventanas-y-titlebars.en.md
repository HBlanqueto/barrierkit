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
| `.fullscreen_bar(FullscreenBar)` | How the overlaid titlebar is drawn in full screen; `Solid` by default ([below](#the-bar-in-full-screen-fullscreenbar)) |

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
The screenshots in this section are the Rótulo window (island sidebar, `Top`
layout unless stated) rendered by the kit in each state.

### The four states with the default layout (`Top`)

| State | How it is entered | What the kit does | How it looks |
|---|---|---|---|
| Normal | — | Titlebar in place, traffic lights in the sidebar. The window edges resize it. On close, the size is saved | ![Normal window](assets/estados/estado-normal.png) |
| Maximized | Green button with `green_button = "zoom"`, double click with `zoom` or `fill`, or `toggle_maximize()` | Same as normal, without resize edges. In MalaysiaDreams the compositor keeps the frame (corners and shadow). On close, the kit saves that it was maximized and reopens it that way | ![Maximized window: same as normal](assets/estados/estado-maximizada.png) |
| Full screen without the focus protocol (another compositor) | Green button with `focus` (default) or `fullscreen`, or `toggle_fullscreen()` | The titlebar stays in place and the traffic lights hide | ![Full screen without the protocol: no traffic lights](assets/estados/estado-completa-sin-protocolo.png) |
| MalaysiaDreams focus mode, bar up | Same; the compositor gives the `fullscreen` state | The titlebar leaves the layout and the content fills the whole window | ![Focus mode with the bar up: content only](assets/estados/estado-foco-barra-arriba.png) |

In MalaysiaDreams, focus mode
([MalaysiaDreams ADRs 0051 and 0054](https://github.com/HBlanqueto/malaysia/tree/main/docs/decisiones))
slides a black bar down when the pointer nears the top. The kit requests
`set_overlay` through `infinity_focus_v1` v2, so the window does not move:
while the bar is down, `titlebar_overlaid()` is true and the titlebar is drawn
over the content, right below the bar, on the background the app chooses
(`FullscreenBar`, below). It appears and disappears with the bar in
`DURATION_BASE`. In full screen, `reveal` is always `Always`.

### The bar in full screen: `FullscreenBar`

Each window chooses how its overlaid titlebar is drawn. The option goes in
`WindowOptions` and applies to the `Titlebar`, the rail and the pill:

```rust
WindowOptions::new("Ondina", "ondina").fullscreen_bar(FullscreenBar::ProgressiveGlass)
```

**By default, every app uses `Solid` ("A · Lisa").** Only Ondina requests
`ProgressiveGlass`. The five come from the "Ondina a pantalla completa"
proposal (A, B, C and E accepted). In the screenshots, the black strip at the
top is the compositor bar, added to the image.

| `FullscreenBar` | What it draws | Where the content starts (`fullscreen_content_top`) | How it looks |
|---|---|---|---|
| `Solid` ("A · Lisa", **default**) | An opaque band in the window color, as tall as the titlebar. It cuts what passes below with a hard edge | Bar + `FULLSCREEN_BAR.content_top` (44) | ![A · Lisa](assets/estados/barra-lisa.png) |
| `Glass` ("B1 · Vidrio en banda") | A band as tall as the titlebar that blurs what is below (`FULLSCREEN_BAND`), with a tint and a line at the bottom | Bar + `FULLSCREEN_BAND.content_top` (44) | ![B1 · Glass band](assets/estados/barra-vidrio-banda.png) |
| `ProgressiveGlass` ("B2 · Vidrio progresivo", Ondina) | Maximum blur at the top that fades out over 96 px, without a line, from edge to edge of the window ([ADR 0057](../decisiones/0057-cristal-y-barra-lateral-sobre-fondo.md)) | Bar + `FULLSCREEN_GLASS.content_top` (74) | ![B2 · Progressive glass](assets/estados/barra-vidrio-progresivo.png) |
| `Scrim` ("C · Degradado") | No band: a veil (`Palette::scrim`) that fades downward over `FULLSCREEN_BAR.scrim_height` (84); the controls float on top | Bar + `FULLSCREEN_BAR.content_top` (44) | ![C · Scrim](assets/estados/barra-degradado.png) |
| `Push` ("E · Empuja") | It covers nothing: the titlebar returns to the layout and pushes the detail column down by the height of the bar and the titlebar. The sidebar does not move | 0: the content moves down by itself | ![E · Push, bar down](assets/estados/barra-empuja.png) |

With `Push` and the bar up, the content reaches the top edge:

![E · Push, bar up](assets/estados/barra-empuja-arriba.png)

Only `Glass` and `ProgressiveGlass` blur. With the others, the rail and the
pill are opaque.

### Focus mode with the bar down, by layout

With the default bar (`Solid`). In these screenshots, the black strip at the
top (28 px) is the compositor bar, added to the image: the app does not draw
it.

| Layout | What comes down | How it looks |
|---|---|---|
| `Top` (default) | The `Titlebar` over the content; the sidebar traffic lights move down by the same amount | ![Top with the bar down](assets/estados/foco-top.png) |
| `Unified` | Same as `Top` | ![Unified with the bar down](assets/estados/foco-unified.png) |
| `Rail` | The rail, with the traffic lights in a column | ![Rail with the bar down](assets/estados/foco-rail.png) |
| `Actions` | The wide rail with its footer of actions | ![Actions with the bar down](assets/estados/foco-actions.png) |
| `Pill` | The traffic-light pill; before that it is hidden | ![Pill with the bar down](assets/estados/foco-pill.png) |

### What the app does

1. If it uses `Titlebar`, `WindowChrome`, `Sidebar` or `PreferencesWindow`, it
   does nothing: every state in the tables already works.
2. It leaves room for the bar in the content under the overlaid titlebar, as
   Ondina does:

   ```rust
   let arriba = try_consume_root_context::<WindowControls>()
       .map_or(0.0, |controls| controls.fullscreen_content_top());
   rect().padding(Gaps::new(arriba, 0.0, 0.0, 0.0)).child(contenido)
   ```

3. If a view covers the titlebar (a video player that fills the window), it
   adds its own traffic lights so the window can be maximized or leave full
   screen without it, as the Marea player does:

   ```rust
   window_traffic_lights(WindowControls::get().status.read().activated, false)
   ```

4. For its own behavior (an immersive mode that hides panels), it reads the
   state and decides:

   ```rust
   let controls = WindowControls::get();
   let inmersivo = controls.status.read().fullscreen;
   rect().maybe_child((!inmersivo).then(panel_de_herramientas))
   ```

5. To offer it in a menu, it calls `toggle_maximize()` or
   `toggle_fullscreen()` from an `Action` (chapter [10](10-app.en.md)).

### Resizing from the edges

With `Client` decoration, the runner reserves a band inside the window of
`WINDOW_RESIZE.edge` (5 px) on each side and `WINDOW_RESIZE.corner` (16 px)
along each corner:

1. Over the band, the cursor turns into the arrow for that edge or corner and
   the app stops receiving the motion.
2. A left-button press starts a compositor resize (`xdg_toplevel.resize`).
3. There is no band when the window is maximized, in full screen, fixed-size or
   server-decorated, nor while the app is dragging something.

### The green button and double click

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

Next: [7. Sidebar and SplitView](07-barra-lateral.en.md).
