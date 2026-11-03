# 8. Sidebar and SplitView

**Goal:** set up a sidebar, choose its style and control how it resizes, hides
and collapses.
**Reader:** anyone who builds an app with side navigation.

## The two pieces

| Piece | Module | What it does |
|---|---|---|
| `Sidebar` | `sidebar` | The content: items, sections, account and traffic lights |
| `SplitView` | `views::split` | The container: sidebar + detail + optional inspector; width, hiding and collapsing |

## The sidebar: `Sidebar`

```rust
Sidebar::new()
    .traffic_lights(true)
    .child(SidebarAccount::new("HB", "Héctor Blanqueto", "Cuenta local"))
    .child(
        SidebarItem::new("Inicio")
            .symbol(Symbol::House, ACCENT_SWATCHES[0].1)
            .selected(true)
            .on_select(move |_| seccion.set(0)),
    )
    .child(
        SidebarSection::new("Biblioteca")
            .expanded(true)
            .child(SidebarItem::new("Descargas").symbol(Symbol::ArrowDown, verde).badge(3)),
    )
```

| Piece | Options |
|---|---|
| `SidebarItem::new(texto)` | `.symbol(Symbol, tinte)` or `.icon(svg)`, `.badge(n)`, `.selected(bool)`, `.emphasized(bool)`, `.on_select(..)` |
| `SidebarSection::new(título)` | `.expanded(bool)`, `.on_toggle(..)`: it collapses with a `DURATION_BASE` animation |
| `SidebarAccount::new(iniciales, nombre, detalle)` | `.selected(bool)`, `.on_select(..)` |
| `Sidebar` | `.traffic_lights(true)` puts the traffic lights inside; `.backdrop(SidebarBackdrop::Tinted)` tints it over a tinted backdrop. With the `Rail`, its first row rises to the titlebar row (chapter [7](07-titlebars.en.md#the-sidebar-with-the-rail)) |

![Selected account row](assets/componentes/sidebar-account-light.png)

## Island or flat

The user decides in `appearance.domain` with `sidebar_style`
([ADR 0041](../decisiones/0041-barra-lateral-en-isla.md)). The app does
nothing:

| `island` (default) | `flat` |
|---|---|
| ![Island sidebar](assets/apps/tengen-formularios-light.png) | ![Flat sidebar](assets/apps/tengen-sidebar-flat-light.png) |

Over a tinted backdrop (chapter [13](13-fondos-y-medios.en.md)), the flat
sidebar shows the backdrop, and the island either shows it or is tinted with
`SidebarBackdrop::Tinted` ([ADR 0057](../decisiones/0057-cristal-y-barra-lateral-sobre-fondo.md)).

## The container: `SplitView`

```rust
SplitView::new(barra, detalle)
    .inspector(Some(inspector.into_element()))
    .sidebar_width(guardado.width)
    .sidebar_visible(guardado.visible)
    .on_layout(move |layout: SidebarLayout| disposicion.set(Some(layout)))
```

| Option | Default | Effect |
|---|---|---|
| `.sidebar_width(f32)` | `SIDEBAR.width` | Initial width |
| `.sidebar_visible(bool)` | `true` | Visible on open |
| `.resizable(bool)` | `true` | With `false` there is no grip: fixed width |
| `.sidebar_range(mín, máx)` | `SPLIT_VIEW.sidebar_min`, `.sidebar_max` (180–320) | Width limits. If they change, the width is clamped to the new range |
| `.inspector(Option<Element>)` | `None` | Right pane of `SPLIT_VIEW.inspector` (240) with a separator |
| `.on_layout(..)` | — | Reports `SidebarLayout { width, visible }` when the grip is released or the sidebar is hidden. Use it to save the layout |

### Resizing

The grip is a `SPLIT_VIEW.grip` (8 px) strip on the right edge of the sidebar:

1. On hover, the cursor changes to `col-resize` and an accent line of
   `grip_line` (2 px) appears, with `grip_line_inset` (18 px) free at the top
   and bottom.
2. While dragging, the width follows the pointer with a `DURATION_FAST_MS`
   (120 ms) spring and stays between the minimum and the maximum. With reduced
   motion, the change is immediate.
3. Tab focuses the grip; ← and → change the width in steps of
   `SPLIT_VIEW.key_step` (10 px).
4. On release, `on_layout` receives the final width.

### Hiding

- The `SidebarToggle` button (`SidebarLeft` symbol) and the `TOGGLE_SIDEBAR`
  action (Ctrl+Alt+S) show or hide it with a `DURATION_BASE` animation.
- From any child: `sidebar_control().map(|c| c.toggle())`.

### Collapsing in narrow windows

Below a window width of `SPLIT_VIEW.collapse_below` (760 px), the sidebar hides
on its own. If the user opens it, it appears on top of the content. When the
window widens, it goes back to its place.

| Wide | Narrow (700 px) |
|---|---|
| ![SplitView with sidebar](assets/apps/tengen-formularios-light.png) | ![Collapsed SplitView](assets/apps/tengen-collapsed-light.png) |

### Trying it

In Tengen App Show, the «Ventana › Barra lateral» ("Window › Sidebar") pane
picks island or flat, turns resizing on or off and switches between three
ranges (Tengen 180–320, Estrecha "narrow" 160–220, Ancha "wide" 240–420):

![«Barra lateral» pane of Tengen App Show](assets/apps/tengen-barra-lateral-light.png)

`PreferencesWindow` passes the same through `.sidebar_resizable(bool)` and
`.sidebar_range(Option<(mín, máx)>)`.

Next: [9. Views](09-vistas.en.md).
