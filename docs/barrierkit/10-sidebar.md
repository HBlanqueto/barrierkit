# 10. Sidebar and split view

**Goal:** build a window with a sidebar that selects what the detail shows, and
control how the sidebar resizes, hides and collapses.
**Scope:** from the sidebar content (`sidebar`) to the container that places it
beside the detail (`views::split`). The titlebar over the detail is in
[9](09-titlebars.md#titlebar); the views inside the detail are in
[11](11-views.md); a ready-made settings window with a sidebar is in
[12](12-patterns.md).
**Reader:** anyone who builds an app with side navigation.
**Input:** a window that draws its own titlebar with `run_window`
([2](02-first-app.md), [8](08-windows.md)). **Output:** a sidebar app whose
page follows the selected item.

## Steps

1. Keep the selected page in state:

   ```rust
   let mut page = use_state(|| 0);
   ```

2. Build one `SidebarItem` per page. It shows the selection the app passes and
   reports a press through `on_select`:

   ```rust
   let item = |index: usize, title: &'static str, glyph: Symbol| {
       SidebarItem::new(title)
           .symbol(glyph)
           .selected(*page.read() == index)
           .on_select(move |_| page.set(index))
   };
   ```

3. Put the items in a `Sidebar`, grouped in `SidebarSection`s if needed:

   ```rust
   let sidebar = Sidebar::new()
       .traffic_lights(true)
       .child(item(0, "Home", Symbol::House))
       .child(
           SidebarSection::new("Library")
               .child(item(1, "Recent", Symbol::Clock))
               .child(item(2, "Downloads", Symbol::ArrowDownCircle)),
       );
   ```

4. Build the detail: a `Titlebar` with `.with_sidebar(true)` and the
   `SidebarToggle` button, then the page:

   ```rust
   let detail = rect()
       .expanded()
       .direction(Direction::Vertical)
       .child(Titlebar::new().with_sidebar(true).title("Home").left(SidebarToggle))
       .child(VStack::new().expanded().child(Text::new("Home")));
   ```

5. Join both in a `SplitView` and return it from the `run_window` closure:

   ```rust
   SplitView::new(sidebar, detail)
   ```

6. Optional: save the layout the user leaves and pass it back on the next
   launch with `.sidebar_width(..)` and `.sidebar_visible(..)`:

   ```rust
   SplitView::new(sidebar, detail)
       .on_layout(move |layout: SidebarLayout| saved.set(Some(layout)))
   ```

7. Optional: put a list between the sidebar and the detail
   ([Content column](#content-column)):

   ```rust
   SplitView::new(sidebar, detail).content(notes_list)
   ```

The names come from `bk::sidebar`, `bk::views::split`,
`bk::symbol::Symbol` and `ctdesign::ACCENT_SWATCHES`.

## Sidebar content

### `Sidebar`

The column of navigation rows. Its children scroll in a
[`ScrollView`](05-controls.md). Its accessibility role is navigation.

| Method | Default | Effect |
|---|---|---|
| `Sidebar::new()` | — | An empty sidebar |
| `.child(view)`, `.children(views)` | — | Adds rows: items, sections, an account or any view |
| `.traffic_lights(bool)` | `false` | Draws the window's traffic lights in the sidebar's top corner. Where they go with each titlebar layout is in [9](09-titlebars.md) |
| `.style(f)` | Tengen | A `SidebarStyle` ([Style](#style)) |
| `.key(key)` | none | Diff key, as on any Freya element |

The sidebar is flat and full height, as in macOS 27 ([ADR 0079](../decisions.md)):
it starts at the window's left edge and runs from the top to the bottom, with
no inset, rounded corner, rim or line. Only its color sets it apart from the
content: `#ededed` against `#ffffff` in light, `#262626` against `#1e1e1e` in
dark. The window's unified title bar (52 px, [9](09-titlebars.md)) runs over
the content only, and the traffic lights sit inside the sidebar at
`(SIDEBAR.traffic_x, SIDEBAR.traffic_y)` = (19, 19). The first row starts at
y = 52, under the bar.

| Part | Measure |
|---|---|
| Width | `SIDEBAR.width` (220, SwiftUI's) |
| Row | `SIDEBAR.item_height` (32); the selection is the width of the sidebar less `SIDEBAR.padding_x` (10) on each side |
| Label inset | From the selection's edge: `SIDEBAR.item_padding_x` (8) before a symbol, `SIDEBAR.item_text_padding_x` (6) before a label without one, so its text starts at x 16 as in SwiftUI |
| Section header | `SIDEBAR.section_height` (19), `SIDEBAR.section_margin_bottom` (13) between sections |
| Selection | `#0270f5` in light, `#007aff` in dark, with white text |

The strip above the first row moves the window when dragged. When the traffic
lights are in the sidebar, the first row starts below them.

#### Glass

With a compositor that offers `infinity_window_style_v1` version 6, as
Malaysia does, the sidebar is a glass panel. The kit leaves its rectangle
transparent and sends it with `set_glass`; the compositor draws the material and
the rim under and over the rows ([ADR 0065](../decisions.md)).
The app does nothing. The kit sends the panel again whenever the sidebar's area
changes.

A window with no sidebar, like an About box, makes the whole window glass with
`WindowGlass::new(content)`. It leaves the window transparent and sends one
panel with the window's corner. Without compositor glass it paints `surfaces.window`.

Without such a compositor, over a [`Backdrop`](15-backdrops-and-media.md), with
reduced transparency or with the accent as window color, the sidebar paints its
own solid fill, as described above.

#### Style

`Sidebar::style`, `SidebarItem::style`, `SidebarSection::style`,
`SidebarAccount::style` and `SymbolTile::style` take a `SidebarStyle`,
`SidebarItemStyle`, `SidebarSectionStyle`, `SidebarAccountStyle` and
`SymbolTileStyle`. The fields are in
[Styles and themes](../../crates/bk/src/styles/sidebar.rs).

```rust
Theme::tengen().style(|s: SidebarItemStyle| SidebarItemStyle {
    shape: Shape::Capsule,
    ..s
})
```

### `SidebarItem`

A row with an optional symbol and a one-line label.

| Method | Default | Effect |
|---|---|---|
| `SidebarItem::new(label)` | — | A row with that text |
| `.symbol(Symbol)` | none | A bare glyph before the label: the accent, the selected ink on the selected row, as a SwiftUI `Label` in a sidebar `List` |
| `.symbol_tile(Symbol, tint)` | none | A [`SymbolTile`](#symboltile) of color `tint` before the label, as System Settings draws it |
| `.selected(bool)` | `false` | Draws the row as selected. The app decides which row is selected |
| `.emphasized(bool)` | `true` | With `false`, the selection is gray instead of accent. Inside a `Sidebar` it turns gray on its own while the keyboard focus is outside the sidebar, as in AppKit |
| `.style(f)` | Tengen | A `SidebarItemStyle` ([Style](#style)) |
| `.on_select(callback)` | none | Called when the row is pressed |
| `.key(key)` | none | Diff key |

**Events:** `on_select(())`. After it, the item closes the sidebar if it floats
over the content (see [Narrow windows](#narrow-windows)).

**Look:** `SIDEBAR.item_height` (32) high, radius `SIDEBAR.item_radius` (8),
text of `SIDEBAR.item_text_size` (13). The selected row has the accent
background and white text of the same weight; hover changes nothing, as on macOS. In an inactive
window the selection turns gray.

### `SidebarSection`

A titled group of rows. It mounts open; a press on its title collapses or
opens it over `DURATION_BASE` (250 ms). A chevron appears on the title on hover
or with the keyboard focus.

| Method | Effect |
|---|---|
| `SidebarSection::new(title)` | A section with that title (11 pt, weight 600) |
| `.child(view)`, `.children(views)` | Adds rows |
| `.style(f)` | A `SidebarSectionStyle` ([Style](#style)) |
| `.key(key)` | Diff key |

### `SidebarAccount`

The account row at the top of the sidebar: an avatar with the initials, the
name in bold and a detail line below. `SETTINGS.account_height` (48) high. It
reports nothing; `.style(f)` takes a `SidebarAccountStyle`.

```rust
SidebarAccount::new("JA", "Jane Appleseed", "Local account")
```

![Account row with an avatar, the name and "Local account"](../../crates/bk/tests/golden/sidebar-account-light.png)

### `SymbolTile`

`SymbolTile::new(glyph, tint)` draws the square tile of a sidebar item:
`SIDEBAR.app_icon` (20) wide, radius `SIDEBAR.item_tile_radius` (4.5), a plate in the user's icon style with
the edge of [app icons](04-appearance-and-tokens.md#app-icons)
(`ctappearance::icon_plate::tile`). By default the plate is `tint` and the
glyph the light ink of `Resolved::on_accent`, which an inactive window keeps:
only the accent turns grey; with dark icons the plate is near black and the
glyph takes `tint`; clear and tinted tiles look like clear and tinted app
icons. `.rim(color)` adds a 1 px inner border and `.style(f)`
takes a `SymbolTileStyle`; both build the tile again, so they come right after
`new`. It is a rect: it takes Freya's layout and style methods. `SidebarItem` uses it, and so do the
search results of [`SettingsWindow`](12-patterns.md#settingswindow).

### `SidebarControl`

`sidebar_control() -> Option<SidebarControl>` gives any view inside a
`SplitView` a handle on its sidebar. Outside a `SplitView` it returns `None`.
`Titlebar` and `SidebarToggle` read it.

| Method | Returns or does |
|---|---|
| `.toggle()` | Shows the sidebar if it is hidden, hides it if it is shown |
| `.dismiss()` | Hides it only if it floats over the content |
| `.occupied()` | The width the docked sidebar takes now, animated; `0.0` while it floats |
| `.settled()` | `true` when it is shown and fully open |

```rust
if let Some(control) = sidebar_control() {
    control.toggle();
}
```

## Split view

### `SplitView`

The container: the sidebar on the left, an optional content column, the detail
in the rest and an optional inspector on the right. It paints the window
background, so the app does not. It is `NavigationSplitView` with a sidebar, an
optional content column and a detail column.

| Method | Default | Effect |
|---|---|---|
| `SplitView::new(sidebar, detail)` | — | Both panes |
| `.content(view)` | none | A column between the sidebar and the detail: see [Content column](#content-column) |
| `.sidebar_width(f32)` | `SIDEBAR.width` (220) | Width on the first render, clamped to the range |
| `.sidebar_visible(bool)` | `true` | Visibility on the first render |
| `.sidebar_range(min, max)` | `SPLIT_VIEW.sidebar_min`, `.sidebar_max` (180, 320) | Width limits. A new range clamps the current width |
| `.resizable(bool)` | `true` | With `false` there is no grip: the width is fixed |
| `.narrow_sidebar(NarrowSidebar)` | `Overlay` | Where the sidebar opens in a narrow window: see [Narrow windows](#narrow-windows) |
| `.inspector(view, presented)` | none | See [Inspector](#inspector) |
| `.background_extension(view)` | none | See [Background extension](#background-extension) |
| `.on_layout(callback)` | none | Reports [`SidebarLayout`](#sidebarlayout) |
| `.style(f)` | Tengen | A `SplitViewStyle`: inset, grip, background, scrim, inspector line and the animations ([Styles and themes](../../crates/bk/src/styles/split_view.rs)) |
| `.key(key)` | none | Diff key |

`sidebar_width` and `sidebar_visible` are starting values. After the first
render the user owns the layout; read it back with `on_layout`.

### Resizing

The grip is a `SPLIT_VIEW.grip` (8 px) strip on the sidebar's right edge. It
exists only with `.resizable(true)` and while the sidebar is fully open.

1. On hover, the cursor becomes `col-resize` and an accent line of
   `SPLIT_VIEW.grip_line` (2 px) appears, `grip_line_inset` (18 px) from the
   top and the bottom.
2. While dragging, the width follows the pointer and stays in the range.
3. Tab focuses the grip; ← and → change the width by `SPLIT_VIEW.key_step`
   (10 px) with a `DURATION_FAST_MS` (120 ms) spring. With reduced motion the
   change is immediate.
4. On release, `on_layout` receives the new width.

### Hiding: `SidebarToggle` and `TOGGLE_SIDEBAR`

`SidebarToggle` is a unit struct: an icon button with the `SidebarLeft` symbol,
on a 36 `ToolGroup` platter, that calls `SidebarControl::toggle`. Its label is
"Show or Hide Sidebar". It is disabled outside a `SplitView`. Put it in the
titlebar's left zone: `Titlebar::new().left(SidebarToggle)`. It takes
`IconButtonStyle`.

While the sidebar is docked the button leaves the bar and sits inside the
sidebar, at its right end, `SIDEBAR.toggle_inset` (7) from the edge, as in
SwiftUI's `NavigationSplitView`; the title then starts 20 from the edge. When
the sidebar hides, the button slides to its place in the bar, after the
traffic lights, and takes its width there. Floating over the content in a
narrow window, the sidebar does not move it.

`TOGGLE_SIDEBAR` is the [`Action`](13-app.md) with the id `toggle-sidebar` and
the shortcut Ctrl+Alt+S. `SplitView` answers its shortcut itself; add the
action to the app's menu to show it there.

Hiding and showing animate over `DURATION_BASE` (250 ms).

### Narrow windows

`NarrowSidebar` decides where the sidebar opens when the window is narrower
than `SPLIT_VIEW.collapse_below` (760 px):

| Variant | The sidebar opens |
|---|---|
| `Overlay` (default) | Floating over the detail, above the `Palette::scrim` veil, without moving the detail. A click on the veil, Escape or an item's `on_select` closes it |
| `Push` | Docked: it narrows the detail |

1. When the window becomes narrow, the sidebar hides by itself.
2. The user opens it with `SidebarToggle` or Ctrl+Alt+S; it appears as
   `NarrowSidebar` says.
3. When the window grows past 760 px again, the sidebar comes back docked and
   visible, even if the user had closed it. This is the rule of
   [`AdwOverlaySplitView`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.4/class.OverlaySplitView.html)
   without `pin-sidebar`.

### `SidebarLayout`

`SidebarLayout { width: f32, visible: bool }`, what `on_layout` reports. It is
`Serialize` and `Deserialize`, so it can be saved as is. `SplitView` reports it
when the width or the visibility changes, never during a drag. A sidebar that
hid by itself in a narrow window counts as visible.

### Content column

`.content(view)` is the middle column of
`NavigationSplitView(sidebar:content:detail:)`. The recipe is
[`three_columns.rs`](../../agents/recipes/three_columns.rs).

- It starts `SIDEBAR.width` wide and has the sidebar's grip, keys and limits
  (`.sidebar_range`, `.resizable`): there is no token for this column, so it
  reuses the sidebar's. A 1 px separator, the inspector's, divides it from the
  detail.
- It does not hide: the sidebar toggle and a narrow window hide the sidebar
  only, as SwiftUI's `doubleColumn` does. Its width is not reported by
  `on_layout`.
- The `Titlebar` belongs to the detail. `SidebarControl::occupied` stays the
  sidebar's width and `SidebarControl::content` is this column's, so the bar
  keeps the toggle at the sidebar's edge and the traffic lights over the
  columns when the sidebar hides. The content column has no bar of its own:
  leave `sidebar_bar_height()` free at its top.

### Inspector

`.inspector(view, presented)` adds a right pane `SPLIT_VIEW.inspector` (270)
wide, the width of
[`NSSplitViewItem(inspectorWithViewController:)`](https://developer.apple.com/documentation/appkit/nssplitviewitem/init(inspectorwithviewcontroller:)),
after a 1 px separator. Changing `presented` opens or closes it over
`DURATION_BASE`, like SwiftUI's `.inspector(isPresented:content:)`.

### Background extension

`.background_extension(view)` paints `view` under the docked sidebar, as macOS
26 `.backgroundExtensionEffect()` does. The view is laid out as wide as the
detail and its first pixel column is stretched to the window edge; Apple
mirrors the view instead. It is not drawn while the sidebar floats.

Next: [11. Views](11-views.md).
