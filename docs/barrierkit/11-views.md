# 11. Views

**Goal:** compose the content of a window from the kit's containers: layout
views, forms, tables, grids, trees, tabs, panes, alerts and navigation.
**Scope:** every module under `bk::views` except `split` (the sidebar
container, in [10](10-sidebar.md)) and `toolbar` (toolbar items, in
[9](09-titlebars.md#toolbar-items)), and `bk::layout`. Single
controls are in [5](05-controls.md), framed text editing in
[6](06-text-and-editing.md), complete windows in [12](12-patterns.md).
**Reader:** anyone who assembles full screens.
**Input:** a window ([2](02-first-app.md)) or the detail of a `SplitView`
([10](10-sidebar.md)). **Output:** a content view that shows data, reports
what the user picks.

## Steps

The task: a file list that shows an empty state, sorts, selects and asks
before deleting.

1. Keep the data, the order and the selection in state:

   ```rust
   let mut files = use_state(Vec::<Vec<String>>::new);
   let sort = use_state(|| None::<SortOrder>);
   let selected = use_state(Vec::<usize>::new);
   ```

2. With no data, show an empty state:

   ```rust
   if files.read().is_empty() {
       return ContentUnavailableView::new("No files", Symbol::Folder).into_element();
   }
   ```

3. Otherwise, build the view and bind the order and the selection. The view
   never changes the data: the app sorts `files` when `sort` changes.

   ```rust
   let table = Table::new("Files", vec!["Name", "Size"])
       .rows(files.read().clone())
       .sort_order(sort)
       .selection(selected);
   ```

4. Ask before a destructive action. Draw the `Alert` while a flag is set and
   clear the flag in both callbacks:

   ```rust
   let mut confirming = use_state(|| false);
   let alert = confirming.read().then(|| {
       Alert::new("Delete these files?", "You cannot undo this action.")
           .confirm("Delete", move |_| {
               files.write().clear();
               confirming.set(false);
           })
           .cancel("Cancel", move |_| confirming.set(false))
   });
   ```

5. Put the table and the alert in the view:

   ```rust
   rect().expanded().child(table).maybe_child(alert).into_element()
   ```

## Layout

`Spacer`, `Frame`, `Grid`, `ZStack`, `Layout` and `ViewThatFits` are in `bk::layout`, and the prelude exports them.
A `VStack` or an `HStack` is a column or a row; these say what a stack cannot.
They are the SwiftUI layout views app authors miss most; without them each app
builds them by hand with `rect()` and measured widths.

### `Spacer`

`Spacer::new()` takes the space that the other children of its `HStack` or
`VStack` leave, and two spacers share it equally. `.min_length(f32)` is the
least it takes (default `SPACE_2`, what SwiftUI gives `Spacer()` on macOS 27.2.1).
A stack that holds a `Spacer` fills its own axis unless it has a size, and its
other axis stays as its children make it. In a `rect()` of your own, a `Spacer`
grows along both axes and the `rect()` needs `.content(Content::flex())`.

```rust
HStack::new()
    .child(Button::new("Reset"))
    .child(Spacer::new())
    .child(Button::new("Save"))
```

### `Frame`

`view.frame()` wraps the view in a `Frame`: a `rect()` with Freya's size
methods (`width`, `height`, `min_width`, `max_width`). `.alignment(ContentAlignment)` places
the view inside it, `CENTER` by default. `ContentAlignment` is SwiftUI's
`Alignment` with its nine constants, from `TOP_LEADING` to `BOTTOM_TRAILING`;
`HorizontalAlignment` and `VerticalAlignment` are its two axes. `Leading` and
`Trailing` are left and right; they swap [right to left](#right-to-left).

```rust
// SwiftUI: Text("Name").frame(width: 100, alignment: .trailing)
Text::new("Name").frame().width(Size::px(100.0)).alignment(ContentAlignment::TRAILING)
```

`frame(maxWidth: .infinity)` is `.width(Size::fill())` in a column. In a row
`Size::fill()` overflows the siblings after it: give the `HStack`
`.content(Content::flex())` and the frame `.width(Size::flex(1.0))`.

### `Grid` and `GridRow`

`Grid::new()` lines up the cells of its rows. A `GridRow` holds the cells of one
row (`.child`, `.children`) and `Grid::row` adds it. Each column is as wide as
its widest cell, each row as tall as its tallest, and a cell sits in its slot
by `ContentAlignment`, `CENTER` by default.

| Method | What it does |
|---|---|
| `Grid::alignment(ContentAlignment)` | Where each cell sits in its slot |
| `Grid::horizontal_spacing(f32)`, `Grid::vertical_spacing(f32)` | The gaps between columns and between rows; both default to `SPACE_2` |
| `GridRow::alignment(VerticalAlignment)` | Overrides the vertical alignment of that row |
| `Grid::child(view)` | A view outside a row: it spans every column, as in SwiftUI |
| `view.grid_cell_columns(n)` | The cell spans `n` columns; the width it needs beyond them is split equally among them |
| `view.grid_column_alignment(HorizontalAlignment)` | Aligns the whole column of that cell; a cell that spans columns ignores it |

The program is [`layout_grid.rs`](../../agents/recipes/layout_grid.rs). Three limits:

1. Cells are content-sized. A cell with `Size::fill()` or `Size::flex` takes the
   width the Grid has available, not its column's: give a separator a width.
2. Torin places a Grid by the size it had before the Grid arranged its cells.
   The width is exact; the height is the tallest column. A parent that centers
   or ends its children places the Grid off when the tallest cells of
   different rows are in different columns. Put the Grid in a parent aligned
   `Start`, or give it a `.height`.
3. There is no `gridCellAnchor`, no hidden row and no baseline alignment: torin
   cannot state a baseline.

### `ZStack`, `overlay` and `background_view`

`ZStack::new()` draws its children on top of one another, the last on top.
It is as large as its largest child, and `.alignment(ContentAlignment)` places
the others in it (`CENTER` by default). `view.overlay(alignment, content)` and
`view.background_view(alignment, content)` draw `content` over or behind the
view; the view alone sizes the result, so a badge larger than its icon does not
grow it. The recipe is [`layout_layers.rs`](../../agents/recipes/layout_layers.rs).

```rust
// SwiftUI: Image(systemName: "bell").overlay(alignment: .topTrailing) { badge }
Image::system(Symbol::Bell).overlay(ContentAlignment::TOP_TRAILING, badge)
```

Three limits ([Custom layout](#custom-layout)):

1. Torin sizes the parent from the first child (the view, for an overlay and a
   background). A `ZStack` whose first child is not the largest is sized right
   after the layout, but a parent that centers it was already placed by the
   first child: put the largest child first.
2. A child that fills (`Size::fill()`) fills what the container was offered,
   not the base view: give the container or the base a width, or use
   `background_style` for a plain fill.
3. A `ZStack` with its own `.width` keeps its children at the start of it.

### Custom layout

`Layout` is SwiftUI's protocol in one call: `place_subviews(proposal, sizes)`
gets the space offered and the size of each subview, and returns a `Placement`
with the size of the container and the origin of each subview in it.
`LayoutContainer::with_layout(layout).child(..)` is the view; `ZStack` is the
`ZStackLayout` one. The program is `layout_layers.rs`, with a flow layout.
Torin cannot give a child its sibling's size or a reverse direction, but it can
move a child after it measures, so a layout is a placement.

Torin measures each subview once, with the space the container has, so a
layout cannot ask a subview for another size (`sizeThatFits` with several
proposals), `layoutValue` or `spacing` preferences. The same parent limit as the
`ZStack` applies: the subview that `sizing_subview()` returns (the first by
default) sizes the container for its parent.

### `ViewThatFits`

`ViewThatFits::new(Axis::Horizontal)` takes candidates with
`.candidate(extent, view)` and shows the first whose `extent` is at most the
width (or height) offered; the last one shows when none fits. It fills the
axis it watches.

SwiftUI measures each candidate's ideal size. Torin cannot measure a view
before it chooses it, and a `Text` would wrap and always fit, so each
candidate states its extent. The first frame offers 0 and shows the last
candidate; the measured size replaces it right after.

### Right to left

`view.layout_direction(LayoutDirection::RightToLeft)` is SwiftUI's
`.environment(\.layoutDirection, .rightToLeft)`. `LayoutDirection` defaults to
`LeftToRight`; the kit does not read the locale. Torin has no reverse
direction, so the direction is a scope that kit views read and mirror. Under it:

| Mirrors | How |
|---|---|
| `HStack`, `Label`, `VStack`, `Frame`, `Text` | a row runs from the right; `Leading` and `Alignment::Start` in a column or a frame are the right edge; a `Text` with a width aligns right; one that hugs its text keeps its alignment |
| `Grid`, `ZStack`, `LayoutContainer` | the columns and the placed origins flip inside the container |
| `Form`, `Row` | the row runs from the right, the chevron points left, the field's text aligns left |
| the six control styles with a layout direction | `Switch`, `Checkbox`, `RadioGroup`, `Slider`, `LevelSlider`, `Stepper` read it, with their labels |

The recipe is [`right_to_left.rs`](../../agents/recipes/right_to_left.rs). Limits:

1. A view mirrors when it is built under the direction (inside a component
   below `.layout_direction`) or when it is already built in the view the
   modifier wraps. A raw `rect()` mirrors only in the second case, and a
   component of the kit that builds its own `rect()` (`List`, `Table`, the
   sidebar, the title bar) does not mirror yet.
2. Padding, margins and absolute positions are physical (Torin has `left` and
   `right`, not leading): swap them when you read the direction.
3. Text keeps Skia's left-to-right base direction: Freya never sets
   `ParagraphStyle::set_text_direction`, so Arabic or Hebrew runs reorder but
   the punctuation of a right-to-left sentence sits on the wrong side.
4. Symbols do not flip (`flipsForRightToLeftLayoutDirection` is absent) except
   the `Form` chevron.

## Forms

### `Form`

The settings layout: sections of rows inside rounded cards. It is SwiftUI's
`Form` with the grouped style.

```rust
let zoom = use_state(|| true);
Form::new()
    .section(
        Section::titled("Windows")
            .row(Row::toggle("Zoom", zoom))
            .row(Row::value("Status", "Normal · 87 %"))
            .row(Row::navigation("Advanced", |_| {})),
    )
    .highlighted(Some("Status".into()))
```

| Method | Effect |
|---|---|
| `Form::new()` | An empty form |
| `.style(FormStyle::columns)` | SwiftUI's `.columns`: no cards or inset, the labels right-aligned in a column as wide as the widest one, the controls 9.5 after it (below) |
| `.section(Section)` | Adds a section |
| `.pinned(Section)` | In a `SettingsWindow` pane, keeps that section under the title bar while the rest scrolls |
| `.highlighted(Option<Cow<'static, str>>)` | Tints the row with that title with the accent at 12 % |

`sections` is a public field: [`SettingsWindow`](12-patterns.md#settingswindow)
reads it to build its search. The form is inset 20 on each side and its cards are at most 704 wide, centered, as in a SwiftUI grouped `Form`; its cards are
10 apart, and a row is its content plus 10 above and below, at least 36 high.
The control of a row stays at the trailing edge at any width, as in a SwiftUI
`Form`; `.stacked()` puts it under the title.

With `FormStyle::columns` a row is its title on the right edge of the label
column and its control or value after `LIST_GROUP.columns_gap` (9.5); rows are
`columns_spacing` (5) apart. The section titles, headers and footers are not
drawn; a row with an empty title and a `Checkbox` control is a checkbox that
starts at the control column.

```rust
Form::new().style(FormStyle::columns).section(
    Section::new()
        .row(Row::new("").control(Checkbox::new("Toggle", toggle)))
        .row(Row::new("Name").control(TextField::new(name, "Name"))),
)
```

### `Section`

| Method | Effect |
|---|---|
| `Section::new()` | A card without a title |
| `Section::titled(title)` | A card with a title above, `HEADLINE` at weight 500 (`LIST_GROUP.header_weight`) |
| `.row(Row)` | Adds a row; rows are separated by a line |
| `.header(view)` | A view in place of the title, such as a title with buttons. A section with only a header draws no card |
| `.illustration(view)` | A full-width view at the top of the card, above the rows |
| `.footer(text)` | A `CAPTION` note below the card |

`title` and `rows` are public fields.

### `Row`

| Constructor | The row shows |
|---|---|
| `Row::new(title)` | Only the title. Add `.control(view)` to put a control on the right |
| `Row::value(title, value)` | Read-only text on the right |
| `Row::navigation(title, on_press)` | A chevron; the whole row is a button |
| `Row::page(title, pane_id)` | A chevron; the row opens that page of a [`SettingsWindow`](12-patterns.md#pane) |
| `Row::destructive(title, on_press)` | The title in red; the whole row is a button |
| `Row::button(title, on_press)` | A bordered `Button` at the start of the row, as a SwiftUI `Button` in a `Form` |
| `Row::toggle(title, State<bool>)` | A mini [`Switch`](05-controls.md#switch-and-checkbox) (36 × 16, the size SwiftUI uses in a form) on the right, with the title as its label |

| Method | Effect |
|---|---|
| `.control(view)` | The control on the right, aligned with the first line of the title; a `TextField` fills the row after the title, with no border and its text ending at the right, as in a SwiftUI `Form`. The other controls take the look of a `Form` row as well: a `PopUpButton` has no bezel and ends in a 20 circle, a `SegmentedControl` sizes its segments to the widest title, a `Stepper` shows no value, a `DatePicker` is an outlined field with no arrows, a `Slider` is 55 % of the row, a `ProgressView` and a `CapacityIndicator` go under a `.stacked()` title with no gap, and a `Checkbox` takes the whole row at the start. A `Switch`, `Checkbox`, `RadioGroup`, `SegmentedControl` and `Slider` on a row draw the accent vibrantly: in the dark it is lifted by (18, -1, 0) |
| `.detail(text)` | An explanation under the title |
| `.symbol(symbol, tint)` | A [`SymbolTile`](10-sidebar.md#symboltile) before the title |
| `.icon(view)` | Any view before the title, such as a device picture; it replaces `.symbol` |
| `.hover_control(view)` | A control before the accessory that shows only while the pointer is over the row, like «Connect» in a Wi-Fi list |
| `.stacked()` | The control goes under the title at any width, as macOS does with radio groups and lists of checkboxes |
| `.keywords(&[..])` | Extra words for the `SettingsWindow` search |

Long titles and details wrap without clipping the control.

### Style

`Form::style` takes a `FormStyle`: the cards, the rows and their texts. The
fields are in [Styles and themes](../../crates/bk/src/styles/form.rs).

```rust
Form::new().section(section).style(|s| FormStyle { section_spacing: 32.0, ..s })
```

## Groups

### `GroupBox`

`GroupBox::new(Option<Cow<'static, str>>).child(view)`: a rounded box (radius
`GROUP_BOX.radius`, 12) with a faint fill and a hairline border, and the label
above it, `GROUP_BOX.label_height` (17) high. It groups related controls
without a form. `NSBox` puts a 12 high title row over its box and starts the
title 10.75 in: `.style(|s| GroupBoxStyle { label_height: 12.0, label_inset: 10.75, ..s })`.

### `DisclosureGroup`

`DisclosureGroup::new(label).expanded(bool).child(view)`: a header row with a
chevron that shows or hides its children, which SwiftUI does not indent
(`DisclosureGroupStyle::indent` is 0). `.expanded` is the starting state; after
that the user owns it.

### Style

`GroupBox::style` and `DisclosureGroup::style` take a
[`GroupBoxStyle`](../../crates/bk/src/styles/group_box.rs) and a
[`DisclosureGroupStyle`](../../crates/bk/src/styles/disclosure_group.rs).

## Tables

### `Table`

Rows of text, or of views, in columns with a header, like `NSTableView`.

| Method | Effect |
|---|---|
| `Table::new(label, columns)` | `label` names it for accessibility; `columns` are the header titles: literals, `tr!` results, `String`s or `Cow`s |
| `.rows(Vec<Vec<String>>)` | One `Vec` of cells per row |
| `.cell(column, \|row\| view)` | Draws `column` as a view in every row: [views in rows and cells](#views-in-rows-and-cells) |
| `.sort_order(State<Option<SortOrder>>)` | Shows the arrow on the sorted column; a click on a header writes the [`SortOrder`](#sortorder) the user asks for |
| `.selection(State<Vec<usize>>)` | The selected rows; a click writes them. Without it rows do not select |
| `.icons(Vec<Symbol>)` | One symbol per row, drawn before the title of the first column: an icon cell |
| `.scroll_bar(bool)` | `false` hides the scroll bar, like `hasVerticalScroller = false` |

1. A click on a header asks for a new order with
   [`next_sort`](#table-helpers). The app sorts its rows.
2. A click selects a row. Ctrl+click or Super+click adds or removes it;
   Shift+click selects the range from the last click
   ([`select`](#select)). ↑ and ↓ move a single selection.
3. The right edge of each header drags to fix the column's width, with a
   minimum of `TABLE.min_column_width` (10, the default `minWidth` of
   `NSTableColumn`). The handle is `TABLE.resize_handle` (6) wide. Columns
   without a fixed width share the rest.
4. A secondary click on the header opens a menu with a checkbox per column to
   show or hide it. One column always stays.

The table keeps column widths and hidden columns itself and does not report
them. Rows are `TABLE.row_height` (24) high, striped, and only the visible
ones are built.

When the rows overflow, the table draws the scroll bar of
[`ScrollView`](05-controls.md#scrolling): the legacy one is a 17 wide strip
beside the rows and the header, starting under the header, so the rows and the
stripes keep 17 of their width free; the overlay one floats over them while
the table scrolls. The bar follows the user's setting like `ScrollView` does,
and `List` and `OutlineView` take it the same way.

The selected row is gray (`SelectedContentInactive`, the unemphasized
selection of `NSTableView`) until the table has focus, then it takes the
selection color with the label in `OnSelectedContent`. `OutlineView` and
`Browser` do the same.

### `List`

One column of rows, text or views, without a header, like SwiftUI's `List` with the
`.inset`, `.plain` and `.bordered` styles. It is a `Table` with a
different `TableStyle`.

| Method | Effect |
|---|---|
| `List::new(label, rows)` | `rows` are the strings of the rows; the default is the inset style |
| `.row(\|index\| view)` | Draws every row as a view: [views in rows and cells](#views-in-rows-and-cells). When the view is a `Label`, rows are 24.5 high and the separator starts at the title, and the symbol is the accent, as in SwiftUI |
| `.selection(State<Vec<usize>>)` | The selected rows; a click writes them. Without it rows do not select |
| `.style(\|_\| TableStyle::plain_list())` | `inset_list` (rows inset 10, rounded selection), `plain_list` (full width) or `bordered_list` (a hairline border) |
| `.style(TableStyle::striped)` | Alternating rows, like `.inset(alternatesRowBackgrounds: true)`; it drops the separators |
| `.scroll_bar(bool)` | `false` hides the scroll bar |
| `.section(title, rows)` | A header of `LIST_SECTION.header_height` (28) over `rows`, as with SwiftUI's `Section`; the sections scroll together, are `LIST_SECTION.gap` (20) apart, and `.selection` numbers the rows across them. Use it with `List::new(label, Vec::new())` |

Rows are separated by a hairline from the start of the text to the end of the
row. A selected row and its neighbors have none.

### Views in rows and cells

`List::row` and `Table::cell` draw a view where the table would draw the text,
like SwiftUI's `List { ForEach(..) { Row($0) } }` and `TableColumn { row in .. }`.
The program is [recipe 23](../../agents/recipes.md#23-views-in-list-rows-and-table-cells).

| Method | Effect |
|---|---|
| `List::row(\|index\| view)` | Draws every row of the list, sections included, as the view |
| `Table::cell(column, \|row\| view)` | Draws the cell of `column` in every row as the view |

1. The closure takes the row index and returns any view. The app keeps its data
   (an `Rc`, or a `Copy` struct of `State`s) and the closure reads the row from
   it. It is a closure, not a view, so the table runs it in the row's scope and
   only for the visible rows; a sectioned list runs it for every row.
2. The strings of `rows` give the row count. The string of a view cell stays its
   accessible name ([Common rules](05-controls.md#common-rules)). A column without `.cell`
   draws its string, so a table can mix text and views.
3. The view runs inside a text scope: `Text`, `Label` and a Freya `label()` take
   the font of the table and the ink of the row, which is `OnSelectedContent`
   while the row is selected and the table has focus. A view of text draws the
   same pixels as the text it replaces.
4. In a sectioned list the index counts across the sections, as `.selection`
   does. Selection stays by index.
5. Rows keep the height of the style, `TableStyle::row_height`, so set it for a
   taller row. A row that takes the height of its content is not available:
   Freya's `VirtualScrollView` needs the height before it builds the row.
6. A press on a view reaches the row and selects it. A control stops it:

```rust
// ✗ a click on the switch also selects the row
.cell(1, |row| Switch::new("On", on[row]))
// ✓ the wrapper stops the press
.cell(1, |row| {
    rect()
        .on_press(|event: Event<PressEventData>| event.stop_propagation())
        .child(Switch::new("On", on[row]))
})
```

7. A closure never compares equal (Freya's `Callback`), so a table with a view
   redraws with its parent, as a [button with a label](05-controls.md#buttons)
   does. A `State` read inside the closure redraws that view alone.

Not measured: the height and the insets of a row that holds a view, and where
the separator starts under an avatar. The rows keep the values of text rows;
the [list row modifiers](#list-row-modifiers) choose only among those.

### `SortOrder`

`SortOrder { column: usize, ascending: bool }`: the column index and the
direction.

### List row modifiers

`List` and `Table` take `ListRowModifiers`, like SwiftUI's `listRow*`. Each method takes a closure of the row index, which in a sectioned `List` counts across the sections. The program is [`list_rows.rs`](../../agents/recipes/list_rows.rs). SwiftUI apps style their rows with these, so the kit takes the same names.

| Method | Effect |
|---|---|
| `.list_row_insets(\|row\| f32)` | Horizontal padding of the cells of that row instead of `TableStyle::padding_x` |
| `.list_row_separator(\|row\| Visibility)` | `Visible` or `Hidden` forces the line under the row; `Automatic` keeps the style's |
| `.list_row_background(\|row\| ShapeStyle)` | Fills the row; a selected row keeps its selection color |

### Table helpers

| Function | Returns |
|---|---|
| `next_sort(current, column)` | The same column flips its direction; another column starts ascending |
| `shown_columns(hidden, count)` | The indices in `0..count` not in `hidden` |
| `toggle_column(hidden, column, count)` | `hidden` with `column` shown again, or hidden if another column stays visible |

### Style

`Table::style` takes a `TableStyle`: the header, the stripes and the
selection. The fields are in
[Styles and themes](../../crates/bk/src/styles/table.rs).

The presets are the styles of `NSTableView` and `NSOutlineView`
([measurements](../research/macos-control-metrics.md#classic-tables-and-outlines)):

| Preset | `NSTableView.Style` | What changes |
|---|---|---|
| `TableStyle::tengen()` | `.inset`, `.automatic` | Rows inset 10, 5 below the header, rounded selection |
| `TableStyle::full_width()` | `.fullWidth` | Rows and stripes from edge to edge, square selection |
| `TableStyle::plain()` | `.plain` | `full_width` with 17 high rows, the cell 8 in and no extra inset in later columns |
| `TableStyle::source_list()` | `.sourceList` | No header, rows inset 10 and 32 high, accent icons |
| `.row_size(ControlSize)` | `rowSizeStyle` | `Small`, `Regular` and `Large` are small, medium and large: the font is 11, 13 and 15 and the icon 14, 16 and 18; only a source list changes its rows, to 24, 32 and 40 |

```rust
Table::new("Files", vec!["Name", "Kind"])
    .rows(rows)
    .icons(vec![Symbol::Document; 3])
    .style(|_| TableStyle::full_width().row_size(ControlSize::Large))
```

## Collections

### `CollectionView`

A wrapping grid of icons with names, like `NSCollectionView`.

```rust
CollectionView::new("Documents")
    .item(icon(), "Notes")
    .item(icon(), "Invoices")
    .selection(selection)
```

| Method | Effect |
|---|---|
| `CollectionView::new(label)` | An empty grid; `label` names it for accessibility |
| `.item(icon, name)` | A `COLLECTION.tile` (92) wide tile with a 48 icon area |
| `.selection(State<Vec<usize>>)` | The selected items; a click writes them. Without it items do not select |

Clicks select as in [`Table`](#table). A plain click on an item already
selected writes nothing.

### `select`

`select(current, index, held: Modifiers) -> Vec<usize>` is the click rule of
`Table`, `CollectionView` and `OutlineView`:

| Held | Result |
|---|---|
| Nothing | Only `index` |
| Ctrl or Super (⌘ or ⌃ on macOS) | `current` with `index` added or removed |
| Shift | The range from the last item of `current` (the anchor, kept last) to `index` |

### Style

`CollectionView::style` takes a `CollectionViewStyle` for its tiles. The
fields are in [Styles and themes](../../crates/bk/src/styles/collection_view.rs).

## Trees

### `TreeNode`

`TreeNode::new(title).icon(Symbol).expanded(bool).child(TreeNode)`: one node of
a tree. Its fields `title`, `icon`, `expanded` and `children` are public. A
node with `expanded(true)` starts open. A path is the list of child indices
from the root: `[0, 1]` is the second child of the first node.

### `OutlineView`

`OutlineView::new(label, nodes).selection(paths)`: the tree
as indented rows with disclosure chevrons, like SwiftUI's `OutlineGroup` in an
inset `List`. Rows are `DISCLOSURE.row_height` (24) high, inset 10 from the
view, and each level indents 13 inside the selection. The view keeps which
nodes are open. `.selection(State<Vec<Vec<usize>>>)` binds the selected paths;
without it the view keeps its own. Ctrl+click and Shift+click work over the visible rows as in
[`select`](#select). A hairline runs under each row from its title to the end
of the row, except in the last top-level node, next to a selected row and
under the last row. `.scroll_bar(bool)` hides the scroll bar.

`OutlineViewStyle::classic()` is `NSOutlineView` with the default cell views:
gray icons, no hairlines and the title 19 after the disclosure column;
`OutlineViewStyle::full_width()` is the same with square rows from edge to
edge, like `NSTableView.Style.fullWidth`.

### `Browser`

`Browser::new(label, nodes).selection(path)`: the tree as columns,
like Finder's column view (`NSBrowser`). Choosing a node with children opens
the next column. Columns are `DISCLOSURE.column_width` (200) wide and scroll
sideways. `.selection(State<Vec<usize>>)` binds the chosen path; without it
the view keeps its own.

### Tree helpers

| Function | Returns |
|---|---|
| `visible_rows(nodes, expanded)` | `(path, depth)` for each row shown when the paths in `expanded` are open |
| `node_at(nodes, path)` | The node at `path`, or `None` |
| `open_paths(nodes)` | The paths of the nodes marked `expanded` |

### Style

`OutlineView::style` and `Browser::style` take the same `OutlineViewStyle`,
so a theme draws both alike. The fields are in
[Styles and themes](../../crates/bk/src/styles/outline_view.rs).

```rust
Theme::tengen().style(|s: OutlineViewStyle| OutlineViewStyle { row_shape: Shape::Capsule, ..s })
```

## Tabs

### `TabView`

Pages chosen from a strip of tabs, like SwiftUI's `TabView` and `NSTabView`:
a segmented strip centered on the top edge of a box that holds the page. Each
tab is as wide as its title plus 28.4 (31 in AppKit, whose titles are
narrower), the chosen one is a raised tab, and the strip is opaque.

```rust
TabView::new("Preferences")
    .tab(Symbol::MagnifyingGlass, "Search", search_page)
    .tab(Symbol::SpeakerWave, "Sound", sound_page)
    .selection(tab)
```

| Method | Effect |
|---|---|
| `TabView::new(label)` | An empty tab view; `label` names the strip for accessibility |
| `.tab(Symbol, title, content)` | Adds a tab and its page; the icon only shows in `TabType::Toolbar` |
| `.selection(State<usize>)` | The selected tab; a click on a tab writes it. Without it the view keeps its own |
| `.size(ControlSize)` | Small and mini strips (20 and 16 high), like `controlSize` |

The view switches pages by itself; `.selection` is for the app to remember or choose the tab.
It fills the space it is given.

### Style

`TabView::style` takes a `TabViewStyle`: the strip and its tabs. Its
`tab_type` says where the tabs are, like `NSTabView.tabViewType`:

| `TabType` | `NSTabView.TabType` | Look |
|---|---|---|
| `Top` | `.topTabsBezelBorder` | The strip straddles the top edge of the bezel box (default) |
| `Bottom` | `.bottomTabsBezelBorder` | The strip straddles the bottom edge |
| `Toolbar` | `NSTabViewController` `.toolbar` | Icon over title, as in a settings window: `TabViewStyle::toolbar()` |
| `NoTabsBezelBorder` | `.noTabsBezelBorder` | The box without a strip |
| `NoTabsLineBorder` | `.noTabsLineBorder` | A square 1 line, no fill |
| `NoTabsNoBorder` | `.noTabsNoBorder` | The page alone |

The left and right strips of `NSTabView` are not drawn. The tab view that SwiftUI hosts draws the same
box and strip. The fields are in
[Styles and themes](../../crates/bk/src/styles/tab_view.rs); the numbers are in
[the measurements](../research/macos-control-metrics.md#tab-views).

```rust
TabView::new("Settings").style(|_| TabViewStyle::toolbar())
```

## Split panes

### `SplitPanes`

Panes in a row or a column with dividers the user drags, like `NSSplitView`. A
pane can be another `SplitPanes`.

```rust
SplitPanes::new("Panes", Direction::Horizontal)
    .pane(list)
    .pane(preview)
    .ratios(ratios.read().clone())
    .on_ratios(move |next: Vec<f32>| ratios.set(next))
```

| Method | Default | Effect |
|---|---|---|
| `SplitPanes::new(label, Direction)` | — | `Horizontal` places panes side by side, `Vertical` stacks them |
| `.pane(view)` | — | Adds a pane |
| `.ratios(Vec<f32>)` | equal | The share of each pane. A list of the wrong length means equal shares |
| `.on_ratios(callback)` | none | Receives the new shares on every pointer move of a drag |
| `.style(f)` | Tengen | A `SplitPanesStyle`: divider width and paint ([Styles and themes](../../crates/bk/src/styles/split_panes.rs)) |

The ratios are controlled: store what `on_ratios` reports and pass it back, or
the divider does not move. A drag changes only the two panes beside the
divider, and neither goes below `SPLIT_PANES.pane_min` (60 px). Dividers are
`SPLIT_PANES.divider` (1 px) thick, the thin divider of `NSSplitView` and the
one of `HSplitView` and `VSplitView`; the pointer grabs them over
`SPLIT_PANES.grab` (5 px).

## Text view

### `TextView`

Multi-line editable text without a frame or scrolling, like an `NSTextView`
inside a custom view. For a framed field with find and spelling, see
[6](06-text-and-editing.md).

```rust
TextView::new(text, MONO.family, 13.0, 16.0, color(Palette::current().label))
    .auto_focus(true)
    .on_key(|press: KeyPress| press.key == Key::Named(NamedKey::Tab))
```

| Method | Default | Effect |
|---|---|---|
| `TextView::new(value, family, size, line_height, ink)` | — | `value: State<String>` holds the text; `ink` is a `Color` |
| `.cursor(Color)` | `ink` | Caret color |
| `.width(Size)`, `.height(Size)` | fill, auto | Size |
| `.auto_focus(bool)` | `false` | Takes the focus when mounted |
| `.on_focus_change(callback)` | none | Receives `true` on focus and `false` on blur |
| `.on_key(callback)` | none | Sees each key first; returning `true` consumes it |

1. Enter breaks the line.
2. Escape and a click outside release the focus.
3. Ctrl or Super with a letter types nothing, except the editing keys A, C,
   X, V, Z and Y.

### `KeyPress`

What `on_key` receives: `key`, `modifiers` and `cursor_utf16`, the caret
position in UTF-16 units.

### Style

`TextView::style` takes a `TextViewStyle`: the paint of the text and the
caret. `ink` and `.cursor(color)` override it before `.style`
([Styles and themes](../../crates/bk/src/styles/text_view.rs)).

## Feedback

Module `views::feedback`. `ContentUnavailableView` is a rect: it takes Freya's
layout and style methods. `Sheet` is a component and takes none
([modifiers on controls](05-controls.md#modifiers-on-controls)).

### `ContentUnavailableView`

`ContentUnavailableView::new(title, symbol)` draws a large faint symbol over a
line of text, for a view with nothing to show, like SwiftUI's
`ContentUnavailableView`. It fills the space it is given and centers its
content, as SwiftUI's does; an app adds no `center()` of its own.

| Call | Adds |
|---|---|
| `ContentUnavailableView::search(query)` | SwiftUI's `.search(text:)`: a magnifying glass, “No Results for …” and a hint, in the app's language |
| `.description(text)` | A second line under the title |
| `.actions(element)` | A row under the description, such as a `Button` |

Each of them and `.style` rebuild the view, so Freya's layout and style methods
come after them. Sizes, gaps and fonts are the `CONTENT_UNAVAILABLE_*` tokens
([4](04-appearance-and-tokens.md)), the frames SwiftUI reports
([coverage](../research/swiftui-appkit-coverage.md)).

### `Sheet`

`Sheet::new(content)` covers the window with the `Palette::scrim` veil (a gray
at 60%, as the window server veils a window under a sheet) and
shows `content` in a dialog centered in the window. The dialog is filled with
the window background (`DIALOG_SOLID_LIGHT` or `DIALOG_SOLID_DARK` with Increase Contrast, and a lighter veil in light) and has radius 16.8
(`CORNER_SHEET.fitted`, the silhouette of a sheet; an `Alert` takes `CORNER_ALERT.fitted`, 26) with a continuous corner, 20 of padding and the window shadow (`SHADOW`, or
`SHADOW_DARK` in dark), so it floats like a window of its own. Draw it while it should show;
remove it to close it.

In a window, the sheet is a window of its own, like an AppKit sheet: a child
toplevel marked modal with `xdg-dialog-v1`, sized to its content (it starts at
`DIALOG_SHEET_WIDTH` × `DIALOG_SHEET_HEIGHT`). The compositor keeps it centered
on the window that opened it and moves it with that window. It is not a
panel: it takes the keyboard, so the window behind deactivates and dims under
the veil. On macOS it is an AppKit sheet (`beginSheet`): AppKit places it and
veils the window, so the kit draws no veil. Without a window, or in a panel
(tests, shell layers), it draws inside the surface over the veil.

While a sheet or an [`Alert`](#alert) shows, the window that opened it is modal
as in AppKit: its content takes no clicks, minimize and zoom still work, and the
close light is disabled. The sheet ignores a close request from the compositor:
only removing it closes it.

`.sheet(is_presented, || content)` on a container is SwiftUI's `sheet(isPresented:content:)`: the
sheet shows while the `State<bool>` is `true`, and the app closes it by writing `false`
(from a button of the sheet, as there is no other way to close one). It has no `onDismiss`
and no `item:` form: nothing closes a sheet but the app. A sheet and its window do not
animate in the kit; the compositor or AppKit draws them.

`.alert(is_presented, || Alert::new(..))` is `alert(_:isPresented:actions:message:)`: the alert
shows while the `State<bool>` is `true`, and every button, Enter and Escape write `false` after
their callback, as SwiftUI dismisses an alert from its actions. `.popover(is_presented, size,
content)` is `popover(isPresented:content:)`: it opens a [`Popover`](07-menus-and-popovers.md#popover)
that points at the view it sits on, writes `false` when the user dismisses it (a click outside) and
closes it when the app writes `false`. `size` is the card's, as in `Popover::new`; the closure runs
each time the popover opens, and nothing opens until the view has a size. `.popover` uses a hook:
call it unconditionally. Neither has `onDismiss`, `item:`, `attachmentAnchor` or `arrowEdge`.
Both take a `State<bool>`, as `sheet` does, because the kit has no `Binding`.
The recipe is [`presentations.rs`](../../agents/recipes/presentations.rs).

### `Alert`

A [`Sheet`](#sheet) with a title, a message and two buttons, like `NSAlert`. With three or more buttons they stack, like `NSAlert` and SwiftUI's `.confirmationDialog`, which has no layout of its own.

```rust
Alert::new("Forget this network?", "You will have to type the password again.")
    .confirm("Forget", move |_| forget())
    .cancel("Cancel", move |_| close())
    .suppression(dont_ask_again)
```

| Method | Default | Effect |
|---|---|---|
| `Alert::new(title, message)` | — | Title in `HEADLINE`, message below |
| `.confirm(label, callback)` | "OK", nothing | The primary button, on the right; Enter presses it |
| `.cancel(label, callback)` | "Cancel", nothing | The other button; Escape presses it |
| `.suppression(State<bool>)` | none | Adds the "Do not show this message again" checkbox bound to the state, like `NSAlert`'s `showsSuppressionButton` |
| `.destructive()` | off | The primary button takes `ButtonStyle::destructive`, like `NSAlert.hasDestructiveAction` |
| `.icon(view)` | none | A `DIALOG_ALERT_ICON` (64) square above the title, the application icon in `NSAlert`; the title starts `DIALOG_ALERT_ICON_GAP` (16) under it |
| `.critical()` | off | With `.icon`, like `NSAlert.Style.critical`: a caution triangle fills the square and the icon shrinks to `DIALOG_ALERT_BADGE` (32) in its bottom right corner. `NSAlert.Style.warning` and `.informational` look like no style, so they have no method |
| `.shows_help(callback)` | none | A help button, 28 round, 12 from the top and the right, like `NSAlert.showsHelp` |
| `.accessory_view(view)` | none | A view under the message, 16 from it and from what follows, like `NSAlert.accessoryView`; it fills the alert's width less 16 on each side |
| `.add_button(label, callback)` | none | An extra button between the confirm and the cancel buttons; with one the buttons stack, 6 apart, confirm first and cancel last |

The alert is `DIALOG_ALERT_WIDTH` (260) wide and splits its two large buttons
evenly, 16 from the sides and the bottom (`DIALOG_ALERT_PADDING`) while the
texts keep the sheet's 20. Title and message are one color and 16 a line, 10
apart (`DIALOG_ALERT_SPACING`). Built by hand, it does not close itself: the
callbacks stop drawing it; `.alert(is_presented, ..)` closes it. Its glass and
its shadow are the compositor's.

### `SheetButtons`

The buttons at the bottom of a [`Sheet`](#sheet) that holds a form: Cancel and
the default button on the right.

```rust
SheetButtons::new("Cancel", move |_| close(), "Save", move |_| save())
    .confirm_disabled(name.is_empty())
    .leading(Button::new("Delete").style(|s| s.borderless()).on_press(move |_| delete()))
```

| Method | Default | Effect |
|---|---|---|
| `SheetButtons::new(cancel, on_cancel, confirm, on_confirm)` | — | Cancel in the plain style and the confirm button as `ButtonStyle::bordered_prominent`, `SPACE_2` apart |
| `.confirm_disabled(bool)` | `false` | Dims the confirm button and ignores its presses |
| `.leading(view)` | none | A view at the left, such as a Delete or Reset button |
| `.large()` | off | Both buttons take `ControlSize::Large` and `DIALOG_BUTTON_WIDTH` (84) of width |
| `.equal_width()` | off | The two buttons split the row evenly, as in an [`Alert`](#alert); `.leading` does not show |

The buttons are plain `Button`s: unlike an [`Alert`](#alert), the row binds no
keys and does not close the sheet. The callbacks do.

### Style

`Sheet::style`, `Alert::style` and `ContentUnavailableView::style` take a
[`SheetStyle`](../../crates/bk/src/styles/sheet.rs), an
[`AlertStyle`](../../crates/bk/src/styles/alert.rs) and a
[`ContentUnavailableViewStyle`](../../crates/bk/src/styles/content_unavailable_view.rs).
An `Alert` draws on the `SheetStyle` of the theme and the containers.

## Navigation

### `History`

`History<T>` keeps a back-and-forward list of pages.

| Method | Effect |
|---|---|
| `History::new(root)` | A history at `root` |
| `.current()` | The page shown |
| `.push(page)` | Goes to `page` and drops the forward pages. Pushing the current page does nothing |
| `.back()`, `.forward()` | Move one step if possible |
| `.can_go_back()`, `.can_go_forward()` | Whether a step is possible |

### `NavigationButtons`

`NavigationButtons::new(State<History<T>>)` draws ‹ and › in a
[`ToolGroup`](05-controls.md#toolbar-controls) capsule. Each button is disabled at its end of
the history. It also answers the shortcuts of `GO_BACK` and `GO_FORWARD`. The
pair is 73 × 36 and has no divider between the arrows, like the navigational
items of an `NSToolbar` ([measures](../research/macos-control-metrics.md#toolbar-items-and-navigation-buttons)).

```rust
let mut history = use_state(|| History::new(Page::Home));
let open_settings = move |_| history.write().push(Page::Settings);
Titlebar::new().left(NavigationButtons::new(history))
```

### `GO_BACK` and `GO_FORWARD`

Two [`Action`](13-app.md#actions-and-shortcuts)s: `go-back` ("Back", Alt+←) and `go-forward`
("Forward", Alt+→).

### Style

`NavigationButtons` takes no style of its own: its group takes
`ToolGroupStyle` and its arrows `IconButtonStyle`
([Styles and themes](../../agents/styles.md#which-controls-take-a-style)).

### `NavigationStack`

`NavigationStack::new(path, root, destination)` is SwiftUI's `NavigationStack`: the
root page, and one page per route of a `NavigationPath<R>` the app keeps in a
`State` ([ADR 0085](../decisions.md) keeps the kit without `Binding`; the path is
a `State`, like `History`).

| Item | What it does |
|---|---|
| `NavigationPath::new()`, `push`, `pop`, `pop_to_root`, `truncate(len)`, `len`, `is_empty` | The routes pushed over the root, last on top |
| `NavigationStack::new(path, root, \|route\| page)` | Draws the root and the page of each route |
| `NavigationLink::new(label, route)` | A button that pushes `route` onto the path of the stack above |
| `use_navigation_path::<R>()` | The path of the stack above, for a page that pops or pushes from code |

A push moves the new page in from the right edge and the page below 30 % of
the width to the left (`NAVIGATION_PARALLAX`), dimmed by a black of 0.13
(`NAVIGATION_DIM`), on a critically damped spring of `SPRING_PUSH_MS`; a pop is
the same reversed. Only the pages in motion are mounted: once the stack
settles only the top page is drawn, so a page rebuilds its state when the path
pops back to it. The edge shadow of UIKit is not drawn.

## Search

### `SearchIndex`

A list of entries searched by title and keywords, ignoring case and accents.

```rust
let mut index = SearchIndex::new();
index.add("wifi", "Wi-Fi", &["network", "wireless"]);
let hits = index.search("WIREL");
```

| Method | Effect |
|---|---|
| `SearchIndex::new()` | An empty index; the key type `K` is any `Clone` |
| `.add(key, title, keywords)` | Adds an entry |
| `.search(query)` | The entries whose folded title or a keyword contains the folded query, in the order added. A blank query finds nothing |

`SettingsWindow` builds one from its panes
([`index`](12-patterns.md#index)).

### `Entry`

`Entry { key, title, keywords }`: one result of `search`.

### Search helpers

| Function | Returns |
|---|---|
| `fold(text)` | `text` in lowercase without accents: combining marks dropped, `ß` → `ss`, `ø` → `o`, `ł` → `l` and similar |
| `word_start(name, query)` | `true` if `query` starts a word of `name` other than the first |

Next: [12. Patterns](12-patterns.md).
