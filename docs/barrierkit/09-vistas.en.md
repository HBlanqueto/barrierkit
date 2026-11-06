# 9. Views

**Goal:** know the layer 2 containers that organize the content.
**Reader:** anyone who assembles complete screens.

## Forms: `Form`, `Section`, `Row`

This is the settings piece: rows grouped in cards.

```rust
Form::new()
    .section(
        Section::titled("Ventanas")
            .row(Row::new("Ampliación").control(Switch::new("Ampliación").on(true)))
            .row(Row::value("Estado", "Normal · 87 %"))
            .row(Row::navigation("Avanzado", |_| {})),
    )
    .highlighted(Some("Estado"))
```

![Form with a control row, a value row and a navigation row](assets/componentes/form-light.png)

| Row | For |
|---|---|
| `Row::new(título).control(..)` | Label and control |
| `Row::value(título, valor)` | Read-only information |
| `Row::navigation(título, al_pulsar)` | Going to a subpage (chevron) |
| `Row::destructive(título, al_pulsar)` | Destructive action, in red |

- `.detail(texto)` adds an explanation under the title; long texts wrap to
  several lines without clipping the control.
- `.keywords(&[..])` feeds the `PreferencesWindow` search.
- `Section::new()` has no title; `.footer(texto)` adds a footer.
- `.highlighted(Some(título))` highlights the row chosen by the search.

![Form with long texts](assets/componentes/form-long-text-light.png)

## Table: `Table`

```rust
Table::new("Archivos", vec!["Nombre", "Tamaño"])
    .rows(filas)
    .sort(Some(SortOrder { column: 0, ascending: true }))
    .selected(Some(1))
    .on_sort(move |orden| criterio.set(Some(orden)))
    .on_select(move |fila| elegida.set(fila))
```

![Table sorted by name with one row selected](assets/componentes/table-light.png)

Clicking a header requests a new order (`next_sort` toggles ascending and
descending); the app sorts its rows.

## Grid: `CollectionView`

```rust
CollectionView::new("Documentos")
    .item(icono(), "Malaysia")
    .item(icono(), "Facturas")
    .selected(vec![1])
    .on_select(move |elegidos: Vec<usize>| seleccion.set(elegidos))
```

![Icon grid with one item selected](assets/componentes/collection-light.png)

It supports multiple selection with a modifier.

## Tabs: `TabView`

```rust
TabView::new("Preferencias")
    .tab(Symbol::MagnifyingGlass, "Buscar", pagina_buscar)
    .tab(Symbol::SpeakerWave, "Sonido", pagina_sonido)
    .selected(0)
```

![Buscar and Sonido tabs](assets/componentes/tab-view-light.png)

## Outline and columns: `OutlineView`, `ColumnBrowser`

```rust
let nodos = vec![TreeNode::new("Documentos").icon(Symbol::Folder).child(TreeNode::new("notas.txt"))];
OutlineView::new("Archivos", nodos.clone())
ColumnBrowser::new("Archivos", nodos)
```

`barrier_kit::views::tree` shows a `Vec<TreeNode>` as a tree with disclosure
triangles (`NSOutlineView`) or as columns, Finder style (`NSBrowser`). Rows
are 24 high and each level indents 13, what `NSOutlineView` measures on macOS
26.6.2. Columns are `DISCLOSURE.column_width` wide (200, an estimate). Both
report through `on_select(path)`, a list of indices from the root.

## Window tabs: `WindowTabs`

```rust
WindowTabs::new(titulos, elegida).on_select(..).on_close(..).on_new(..)
```

The tab bar of a tabbed `NSWindow`: capsules that share the width, the chosen
one raised, with the X on hover and a + at the end. Its measurements are an
estimate. The app keeps each tab's content; tabs cannot be dragged to another
window.

## Toolbar: search and overflow

`compact::ToolbarSearch::new(text, "Buscar").collapsed(narrow)` is the toolbar
search field: with `collapsed(true)` it stays a magnifying glass that opens
when pressed and closes when it loses focus empty. `ToolbarOverflow::new(items)`
is the » button that gathers what does not fit into a menu; the app shows it
instead of those items when the `Breakpoint` is `Compact`. Both go in
`Titlebar::right`.

## States, sheets and alerts

```rust
empty_state(Symbol::MagnifyingGlass, "Sin resultados")
sheet(contenido)
Alert::new("¿Olvidar esta red?", "Tendrás que volver a escribir la contraseña.")
    .confirm("Olvidar", |_| {})
    .cancel("Cancelar", |_| {})
```

| Empty | Alert |
|---|---|
| ![Empty state](assets/componentes/empty-state-light.png) | ![Alert with a confirm button](assets/componentes/alert-light.png) |

`sheet` shows the content as a centered dialog over a scrim
(`Palette::scrim`) that covers the window. The dialog has radius 26 with a
continuous corner, like macOS 26 sheets and alerts. `Alert` is 260 wide,
splits its two large buttons (28, capsule) evenly, confirms with Enter and
cancels with Esc.

## Navigation: `History`

```rust
let historial = use_state(|| History::new(Seccion::Inicio));
historial.write().push(Seccion::Ajustes);
navigation_buttons(historial)
```

`History<T>` keeps the back/forward stack. `navigation_buttons` draws ‹ and ›
in a `ToolGroup` capsule with a divider and responds to the `GO_BACK` (Alt+←) and `GO_FORWARD` (Alt+→) actions.

## Search: `SearchIndex`

```rust
let mut indice = SearchIndex::new();
indice.add("wifi", "Wi-Fi", &["red", "inalámbrica"]);
let resultados = indice.search("inalam");
```

It searches ignoring case and accents. `PreferencesWindow` builds it on its
own from the titles and the `keywords` of each `Row`.

## Adaptive layout

```rust
let adaptable = use_breakpoint();
match adaptable.current() {
    Breakpoint::Wide => tres_columnas(),
    Breakpoint::Medium => dos_columnas(),
    Breakpoint::Compact => una_columna(),
}
```

The breakpoints are `BREAKPOINT.medium` (620) and `BREAKPOINT.wide` (900).
`adaptable.attach(rect())` measures the element it is attached to. Ondina uses
it to go from three columns to one (chapter [15](15-apps-de-prueba.en.md)).

Next: [10. Patterns](10-patrones.en.md).
