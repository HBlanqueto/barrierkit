# 7. Titlebars

**Goal:** choose an app's titlebar and know how it looks in each state:
normal, unfocused, maximized, full screen and MalaysiaDreams focus mode.
**Reader:** anyone who designs an app's window.

## What there is

| Piece | For | Default |
|---|---|---|
| `Titlebar` | The horizontal CSD titlebar: traffic lights, left and right zones, title | With a sidebar when `.with_sidebar(true)` is passed |
| `TrafficLights` | Close, minimize and maximize | Square, `traffic_lights` colors |
| `WindowChrome` + `ChromeStyle` | Two layouts: `Top` and `Rail` (normal or unified); both accept the app's own items | `Top` |
| `FullscreenBar` | How the overlaid titlebar is drawn in full screen | `Solid` ("A · Lisa") |

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

### Variants in a normal window

The same `Titlebar` changes by itself with the sidebar, its style and focus.
The app chooses none of these variants: they come from `with_sidebar`, from
`sidebar_style` in `appearance.domain` and from whether the window is active.

| Variant | How it looks |
|---|---|
| With an island sidebar (default): the traffic lights go in the sidebar; the bar has no line | ![Titlebar with an island sidebar](assets/estados/titlebar-isla.png) |
| With an island sidebar, unfocused window: traffic lights and title in gray | ![Titlebar with an island sidebar, unfocused](assets/estados/titlebar-isla-inactiva.png) |
| With a flat sidebar: a `TITLEBAR_CLIENT_HEIGHT` bar with a line at the bottom | ![Titlebar with a flat sidebar](assets/estados/titlebar-plana.png) |
| Without a sidebar: traffic lights in the bar and the title centered between two symmetric zones | ![Titlebar without a sidebar](assets/estados/titlebar-sin-barra.png) |
| Without a sidebar, unfocused window: `titlebar_inactive` background and gray traffic lights | ![Titlebar without a sidebar, unfocused](assets/estados/titlebar-sin-barra-inactiva.png) |

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

## Two titlebar types: `WindowChrome`

`chrome::WindowChrome::new(estilo, contenido)` wraps the window and places the
traffic lights, the title and the rail items according to a `ChromeStyle`:

```rust
let estilo = ChromeStyle {
    layout: TitlebarLayout::Rail,
    rail_width: TITLEBAR_RAIL.width,
    round_lights: false,
    rail_style: RailStyle::Normal,
    reveal: Reveal::Always,
    resting_edge: RestingEdge::None,
};
WindowChrome::new(estilo, SplitView::new(barra, detalle))
    .rail_top(arriba)
    .rail_bottom(abajo)
```

| `TitlebarLayout` | What it is | Screenshot (Tengen App Show) |
|---|---|---|
| `Top` (default) | Horizontal titlebar at the top, with items on the left and the right (`Titlebar::left` and `right`) | ![Top layout](assets/apps/tengen-titlebar-top.png) |
| `Rail` | Vertical titlebar on the left, with the traffic lights in a column, the title rotated 90° and items at the top and the bottom (`rail_top` and `rail_bottom`) | ![Rail layout](assets/apps/tengen-titlebar-rail.png) |

The `Rail` has two styles:

| `RailStyle` | What it is | Screenshot (Tengen App Show) |
|---|---|---|
| `Normal` (default) | Its own strip on the left of the window | ![Normal rail](assets/apps/tengen-titlebar-rail.png) |
| `Unified` | No strip: the column goes inside the sidebar. Without a visible sidebar it is drawn as `Normal` | ![Unified rail](assets/apps/tengen-titlebar-unified.png) |

### Rail items

Just as the `Top` `Titlebar` holds buttons or other items on the left and the
right, the `Rail` holds them vertically:

| Method | Where it goes |
|---|---|
| `WindowChrome::rail_top(items)` | Below the traffic lights |
| `WindowChrome::rail_bottom(items)` | At the foot of the rail |

With any item, the rail widens to `TITLEBAR_RAIL.actions_width` (34). In
`Unified` the items go in the sidebar column, in the same places.

| `Normal` with items | `Unified` with items |
|---|---|
| ![Normal rail with items at the top and the bottom](assets/estados/rail-elementos.png) | ![Unified rail with items at the top and the bottom](assets/estados/rail-unificado-elementos.png) |

### The sidebar with the `Rail`

With the `Rail`, the traffic lights leave the sidebar and free its top. The
`Sidebar` then raises its first row (the search field in `PreferencesWindow`) to
the titlebar row, centered with its buttons. The measure comes from
`sidebar_bar_metrics()` and `TEXT_FIELD.height`, so it holds for island and flat.

When the layout switches from `Top` to `Rail`, or back, the row slides with
`DURATION_BASE` and a cubic ease-out curve. With reduced motion, it jumps. In
full screen it does not move down with the compositor bar, like the rest of the
window with the `Rail`.

| `Top`: the search field, below the traffic lights | `Rail`: the search field, in the titlebar row |
|---|---|
| ![Sidebar with Top](assets/estados/barra-lateral-top.png) | ![Sidebar with Rail](assets/estados/barra-lateral-rail.png) |

### `ChromeStyle` fields

| Field | Values | Effect |
|---|---|---|
| `rail_width` | `TITLEBAR_RAIL.width` or `.width_wide` | Width of the rail without items |
| `round_lights` | `bool` | Round or square (default) traffic lights |
| `rail_style` | `Normal`, `Unified` | Its own strip or a column inside the sidebar |
| `reveal` | `Always`, `Drawer`, `Push`, `Corner`, `Popup` | Always visible, or appears when the pointer nears: as a drawer on top, pushing the content, only in the corner, or as a popup pill with the traffic lights |
| `resting_edge` | `None`, `Line`, `Accent` | What stays visible while hidden: nothing, a line or an accent line |

Some combinations make no sense and correct themselves
(`ChromeStyle::effective`): a unified `Rail` with `Drawer`, `Push` or `Popup`
becomes `Corner`; in full screen, `reveal` is always `Always`.

The Titlebar, Pantalla completa ("Full screen") and Barra lateral ("Sidebar")
panes of Tengen App Show (chapter [15](15-apps-de-prueba.en.md)) test every
combination live.

## Maximized, full screen and focus mode

The state is read as chapter [6](06-ventanas.en.md#window-state) explains.
The screenshots in this section are a test window (island sidebar, `Top`
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
`WindowOptions` and applies to the whole overlaid titlebar:

```rust
WindowOptions::new("Ondina", "ondina").fullscreen_bar(FullscreenBar::ProgressiveGlass)
```

**By default, every app uses `Solid` ("A · Lisa").** Only Ondina requests
`ProgressiveGlass`. The three come from the "Ondina a pantalla completa"
proposal; "C · Degradado" was dropped. In the screenshots, the black strip at the
top is the compositor bar, added to the image.

| `FullscreenBar` | What it draws | Where the content starts (`fullscreen_content_top`) | How it looks |
|---|---|---|---|
| `Solid` ("A · Lisa", **default**) | An opaque band in the window color, as tall as the titlebar. It cuts what passes below with a hard edge | Bar + `FULLSCREEN_BAR.content_top` (44) | ![A · Lisa](assets/estados/barra-lisa.png) |
| `Glass` ("B1 · Vidrio en banda") | A band as tall as the titlebar, from edge to edge of the window and over the sidebar, that blurs what is below (`FULLSCREEN_BAND`), with a tint and a line at the bottom | Bar + `FULLSCREEN_BAND.content_top` (44) | ![B1 · Glass band](assets/estados/barra-vidrio-banda.png) |
| `ProgressiveGlass` ("B2 · Vidrio progresivo", Ondina) | Maximum blur at the top that fades out over 96 px, without a line, from edge to edge of the window and over the sidebar ([ADR 0057](../decisiones/0057-cristal-y-barra-lateral-sobre-fondo.en.md)) | Bar + `FULLSCREEN_GLASS.content_top` (74) | ![B2 · Progressive glass](assets/estados/barra-vidrio-progresivo.png) |

Only `Glass` and `ProgressiveGlass` blur.

#### Pushing the content: `fullscreen_push`

"E · Empuja" is not another bar but an option that combines with any of the
three. It is off by default:

```rust
WindowOptions::new("Ondina", "ondina")
    .fullscreen_bar(FullscreenBar::ProgressiveGlass)
    .fullscreen_push(true)
```

Both are the initial value. The app changes them live through
`WindowControls::get().fullscreen_bar` and `.fullscreen_push`, two `State`s.

With it, the bar covers nothing: all the window content, sidebar included,
moves down by the height of the compositor bar and the titlebar
(`WindowControls::push_offset`). The traffic lights move down with the titlebar
and `fullscreen_content_top` is 0.

| Combination | How it looks |
|---|---|
| `Solid` + `fullscreen_push(true)` | ![A · Lisa pushing the content](assets/estados/barra-empuja.png) |
| `ProgressiveGlass` + `fullscreen_push(true)` | ![B2 · Progressive glass pushing the content](assets/estados/barra-empuja-vidrio-progresivo.png) |
| Bar up: the content reaches the top edge | ![Push with the bar up](assets/estados/barra-empuja-arriba.png) |

#### With the `Rail`

With the `Rail`, the top titlebar neither retracts nor overlays: it stays in
place, as in the normal window, and nothing moves down with the compositor bar.
Only the rail hides while the bar is up and, when the bar comes down, slides in
from the left with the same bar, vertically, with its traffic lights below the
bar:

| `FullscreenBar` | The rail | How it looks |
|---|---|---|
| `Solid` ("A · Lisa", **default**) | An opaque strip as wide as the rail, over the content | ![Rail with A · Lisa](assets/estados/rail-barra-lisa.png) |
| `Glass` ("B1 · Vidrio en banda") | A strip as wide as the rail that blurs what is below, with a line on the inner edge | ![Rail with B1 · Glass band](assets/estados/rail-barra-vidrio-banda.png) |
| `ProgressiveGlass` ("B2 · Vidrio progresivo") | Maximum blur at the window edge that fades inward over 96 px | ![Rail with B2 · Progressive glass](assets/estados/rail-barra-vidrio-progresivo.png) |
| Any + `fullscreen_push(true)` | The rail returns to the layout and pushes the content sideways by its width | ![Rail pushing the content](assets/estados/rail-barra-empuja.png) |

### Focus mode with the bar down, by layout

**The layout is kept when the window is maximized and in full screen**, also
with an island sidebar: the `Rail` stays on the left.

| `Rail` with an island, maximized | `Rail` with an island, focus mode |
|---|---|
| ![Rail with an island and the window maximized](assets/estados/isla-rail-maximizada.png) | ![Rail with an island in focus mode](assets/estados/isla-rail-foco.png) |

The screenshots in this table use the flat sidebar (except `Top`) and the default bar (`Solid`). The
black strip at the top (28 px) is the compositor bar, added to the image: the
app does not draw it.

| Layout | What comes down | How it looks |
|---|---|---|
| `Top` (default) | The `Titlebar` over the content; the sidebar traffic lights move down by the same amount | ![Top with the bar down](assets/estados/foco-top.png) |
| Unified `Rail` | The sidebar column; the traffic lights appear with the bar, below it | ![Unified rail with the bar down](assets/estados/foco-unified.png) |
| `Rail` | The rail slides in from the left with the chosen bar (above, "With the `Rail`"); its traffic lights sit below the compositor bar | ![Rail with the bar down](assets/estados/rail-barra-lisa.png) |
| `Rail` with items | The same, with its items | ![Rail with items and the bar down](assets/estados/foco-rail-elementos.png) |

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
   `toggle_fullscreen()` from an `Action` (chapter [11](11-app.en.md)).

Next: [8. Sidebar and SplitView](08-barra-lateral.en.md).
