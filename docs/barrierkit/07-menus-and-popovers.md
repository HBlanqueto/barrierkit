# 7. Menus and popovers

**Goal:** show a context menu, open a popover and add a tooltip.
**Scope:** menu items (`MenuItem`), the menu list (`MenuView`), its keyboard,
search, shortcuts, mnemonics and submenus; the view modifiers
(`.context_menu`, `.help`, `.popover`, `.focus_ring`) and `Popover`. The menu bar menu
(`MainMenu`) is in [13. App](13-app.md). The pop-up button (`PopUpButton`) is
a control: [5. Controls](05-controls.md).
Popup surfaces in general (`Popups`, `PopupSpec`) are in
[8. Windows](08-windows.md#popups).
**Reader:** anyone who adds a menu, a popover or a tooltip to an app.
**Input:** a window that runs with `Application` or `run_window`
([2. Your first app](02-first-app.md)). **Output:** a view with a context
menu, a button that opens a popover and a control with a tooltip.

## Steps

1. Build the items. Each action has an `id` that comes back when it is chosen:

   ```rust
   use bk::menu::{CheckState, MenuItem};

   let items = vec![
       MenuItem::action("open", tr!("open")).with_shortcut("Ctrl+O"),
       MenuItem::action("hidden", tr!("hidden-files")).with_check(CheckState::Checkbox(true)),
       MenuItem::Separator,
       MenuItem::submenu(tr!("share"), vec![MenuItem::action("mail", tr!("mail"))]),
   ];
   ```

2. Call `.context_menu` on the element that receives the secondary click:

   ```rust
   Text::new(tr!("file-name")).context_menu(items, |id: String| {
       // run the action `id`
   })
   ```

3. For a popover, keep a `State<bool>` and call `.popover` on the view it
   points at ([11. Views](11-views.md#sheet)):

   ```rust
   let mut showing = use_state(|| false);
   rect()
       .child(Button::new(tr!("info")).on_press(move |_| showing.set(true)))
       .popover(showing, (264.0, 56.0), || {
           rect().padding(12.0).child(Text::new(tr!("hello"))).into()
       })
   ```

4. Add a tooltip with `.help`. It works on a control as it does on `rect()`:

   ```rust
   use bk::compact::IconButton;

   IconButton::new(Symbol::Plus, tr!("add")).help(tr!("add-folder"))
   ```

## Menu items

`MenuItem` (`bk::menu`) is an enum. Build it with the constructors
and adjust it with the `with_*` methods; a method that does not apply to the
variant does nothing.

| Constructor | Variant | What it is |
|---|---|---|
| `MenuItem::action(id, label)` | `Action` | A command. Enabled, no check, no icon |
| `MenuItem::submenu(label, items)` | `Submenu` | A row that opens `items` in a child menu |
| `MenuItem::header(label)` | `Header` | A section title; not selectable. `Header { label, count: Some(n) }` adds a count at the right |
| `MenuItem::Separator` | `Separator` | A line between groups |
| `MenuItem::placeholder(label)` | `Action` | A disabled action with an empty `id`, for rows such as "No Items" |
| `MenuItem::palette(items)` | `Palette` | The actions in `items`, drawn as one horizontal row of icons, like `NSMenu.PresentationStyle.palette`. ← and → walk it; Enter picks |
| `MenuItem::SearchField { placeholder }` | `SearchField` | The search row of a searchable menu ([Search](#search)) |
| — | `SearchResult` | A row the menu builds for a search match; apps do not create it |

| Method | Applies to | Effect |
|---|---|---|
| `.with_shortcut(text)` | `Action` | Shows the shortcut at the right ([Shortcuts](#shortcuts)). It only shows it: the window menu or the app handles the key |
| `.with_enabled(bool)` | `Action`, `Submenu` | A disabled row is gray and skipped by the keyboard |
| `.with_check(CheckState)` | `Action` | `None` (default), `Checkbox(bool)`, `Radio(bool)`, `Mixed` or `Minimized`: a check mark, a dot, a dash or the diamond of a minimized window in the leading column. The check, dash and the submenu chevron are strokes of the label color, as in AppKit |
| `.with_icon(MenuIcon)` | `Action`, `Submenu` | An icon before the label |
| `.with_badge(text)` | `Action` | A counter at the right, before the shortcut, like `NSMenuItemBadge` |
| `.with_id(id)` | `Submenu` | Gives the submenu an id, so its contents can load on open ([Submenus](#submenus)) |
| `.label()` | all | The label, or `None` for `Separator`, `Palette` and `SearchField` |

`MenuIcon` has three variants:

| Variant | Build | What it draws |
|---|---|---|
| `Svg(Arc<str>)` | `MenuIcon::svg(text)` | An SVG, tinted with the row color. It is as tall as the other icons and keeps the width over height of its root `width` and `height` |
| `Image(Arc<[u8]>)` | `MenuIcon::image(bytes)` | Encoded image bytes. Returns `None` above `MenuIcon::MAX_IMAGE_BYTES` (64 KiB). The row has no icon if a side is over `MenuIcon::MAX_IMAGE_SIDE` (1024 px) |
| `Swatch(Rgba)` | `MenuIcon::Swatch(color)` | A color circle |

The leading column for check marks exists only if some row uses it. If any row
has an icon, every row of the menu reserves the icon column, so the titles of
the rows without one line up with the rest; a checked row keeps its icon. The
icon column fits the widest icon of the menu, as in AppKit, and the icon has the
text color of its row. The icon is `MENU_ICON_TEXT_GAP` (8) before the title.

An app action from [13. App](13-app.md) becomes an item with
`action.item(enabled)` or `action.item_with(enabled, check)`.

## Menu view

`MenuView` draws a list of items. `.context_menu` and `PopUpButton` create it
for you; build one directly only for your own menu surface.

```rust
MenuView::new(items, |id: String| { /* chosen */ }, || { /* close every level */ })
    .with_surface(SurfaceKind::Toplevel)
```

`MenuView::new(items, on_activate, on_dismiss_all)` reports the chosen
action's `id` through `on_activate`, then calls `on_dismiss_all`.

| Method | Effect |
|---|---|
| `.with_surface(SurfaceKind)` | Palette for a window (`Toplevel`) or a shell surface (`LayerShell`, default). `SurfaceKind` is in `bk::theme` |
| `.with_popover_metrics()` | Uses the `MENU_POPOVER` sizes (24 px rows, 4 px padding, 180 px minimum width), the look of an app menu on macOS |
| `.with_fixed_palette(MenuPalette)` | A fixed palette that ignores the appearance, such as `MENU_LIGHT` or `MENU_DARK` |
| `.with_items_source(f)` | Reads the items from `f` on every render instead of the fixed list |
| `.with_focused_index(Some(i))` | The row focused when it opens |
| `.with_popup_spec(spec)` | The `PopupSpec` the menu was opened with; the menu uses its anchor to place revealed menus |
| `.with_open_submenu(f)`, `.with_submenu_source(f)` | Lazy submenus ([Submenus](#submenus)) |
| `.with_searchable(f)`, `.with_help_topics(f)`, `.with_search_icons(menu, help)`, `.with_reveal_roots(f)`, `.with_query_state(state)` | Search ([Search](#search)) |
| `.with_navigation(f)` | ← and → at the edge call `f(MenuNavigation::Prev)` or `f(MenuNavigation::Next)`, so a menu bar moves to the neighbouring menu |
| `.style(\|s\| ..)` | Changes the `MenuStyle` of this menu and its submenus ([Style](#style)) |

`MenuView::preferred_size(&items, palette)` returns the `(width, height)` the
items need; open the popup with that size. Long labels are cut in the middle
with "…" at the palette's maximum width. An empty list shows a disabled
"No Items" row.

The keyboard works while the menu is open:

| Key | Effect |
|---|---|
| ↑, ↓ | Previous or next selectable row; headers, separators and disabled rows are skipped |
| Home, End | First or last selectable row |
| Enter, Space | Chooses the focused action or opens the focused submenu. Space types into the search while there is a query |
| → | Opens the focused submenu; on a plain row with navigation, `Next` |
| ← | Closes this menu; with navigation, `Prev` instead |
| Esc | Clears the search if there is one; otherwise closes this menu |
| Alt (held) | Underlines the mnemonics ([Mnemonics](#mnemonics)) |

A menu taller than its popup shows scroll arrows at the ends. Hovering one
scrolls a third of a row every `MENU_SCROLL_TICK_MS` (30 ms); the wheel also
scrolls.

### Style

Every menu, pop-up menu and popover draws from a
[`MenuStyle`](../../agents/styles.md#menustyle), resolved through the theme and the
container styles of the view that opened it:

```rust
VStack::new()
    .child(PopUpButton::new("Sort", sort).item("Name").item("Date"))
    .control_style(|s: MenuStyle| MenuStyle { item_radius: 0.0, ..s })
```

## Shortcuts

A shortcut is text: modifiers and a key joined with `+`, in any case.

| Part | Accepted names | Shown as |
|---|---|---|
| Control | `Ctrl`, `Control`, `Primary` | ⌃ |
| Alt | `Alt`, `Option`, `Opt`, `Mod1` | ⌥ |
| Shift | `Shift` | ⇧ |
| Super | `Super`, `Meta`, `Cmd`, `Command` | ❖ |
| Named keys | X keysym names or aliases: `Escape`/`Esc`, `Return`/`Enter`, `Tab`, `space`, `BackSpace`, `Delete`, `Left`, `Right`, `Up`, `Down`, `Home`, `End`, `Page_Up`, `Page_Down`, `comma`, `period`, `minus`, `plus`, `equal`, `slash`, `question`, `backslash`, `semicolon`, `apostrophe`, `bracketleft`, `bracketright` | ⎋ ↩ ⇥ ␣ ⌫ ⌦ ← → ↑ ↓ ↖ ↘ ⇞ ⇟ and the character |
| Any other single character | itself | in upper case |

Modifiers are always shown in the order ⌃ ⌥ ⇧ ❖.

| Function | What it does |
|---|---|
| `format_shortcut(text)` | `"Ctrl+Shift+G"` → `"⌃⇧G"` |
| `shortcut_matches(text, modifiers, &key)` | `true` when a key event is that shortcut. Only Control, Alt, Shift and Super count |

## Mnemonics

A mnemonic is the letter that Alt underlines. Put `MNEMONIC_MARK`
(U+2063, invisible separator) right before that letter in the label:

```rust
let label = format!("{}Open", bk::menu::MNEMONIC_MARK);
```

While Alt is held, the menu underlines the letter after the mark. The mark is
never drawn, and search ignores it. Pressing the letter does not choose the
row. `strip_mnemonic(label)` returns the label without the mark.

## Submenus

A `Submenu` row opens its items in a child popup, to the right or, when there
is no room, to the left. The pointer opens it after `MENU_SUBMENU_INTENT_MS`
(300 ms). While the pointer moves towards an open submenu, the rows it crosses
do not close it. Submenus open only when the `MenuView` lives in a popup.

For contents that arrive later (a dbusmenu `AboutToShow`, for example):

1. Give the submenu an id: `MenuItem::submenu(label, vec![]).with_id("recent")`.
2. `.with_open_submenu(|id| …)` is called with that id when it opens. Start
   loading there.
3. `.with_submenu_source(|id| …)` returns `Some(items)` once they exist. The
   child re-reads it on every render.
4. Until then the child shows a disabled "Loading…" row.
   `submenu_contents(loading, &items)` gives the same result for your own
   lists.

## Search

A menu with `.with_searchable(f)` filters while the user types, as the Help
menu on macOS does. The Malaysia menu bar uses it; most apps do not need it.

1. `f` returns one `SearchableEntry` per item that can be found:

   | Field | What it is |
   |---|---|
   | `item` | The `MenuItem` the result activates |
   | `location` | `Some(SearchLocation { title_index, indices })`: the top menu and the row indices that lead to the item, or `None` |

2. `.with_help_topics(f)` adds a second list of entries, shown under a "Help
   Topics" header.
3. Typing adds to the query; Backspace removes the last character; Esc clears
   it. A `SearchField` row shows the query in a capsule with no focus ring, as
   on macOS; the menu adds one at the top if the items have none.
4. Matches are ranked: label starts with the query, then a word starts with
   it, then it appears anywhere; disabled rows go after enabled ones. The menu
   shows up to `MenuView::MAX_MENU_ROWS` (9) menu results and
   `MenuView::MAX_HELP_ROWS` (5) help results, and ↑ ↓ scroll beyond them.
5. With `.with_reveal_roots(f)`, `f` returns the items of every top menu. When
   a menu result with a `location` is focused, the menu opens that top menu to
   its left, with the item focused, if there is room.
6. `.with_query_state(state)` shares the query (`SearchQuery { text, selected }`)
   through an `Rc<RefCell<SearchQuery>>`, so it survives when the menu is
   rebuilt.
7. `.with_search_icons(menu, help)` sets the SVG icons of the two result
   kinds. `search_icon_svg(template, text, accent)` fills the `{{text}}` and
   `{{accent}}` placeholders of a template with two colors.

`normalize_query(text)` prepares text the way the search compares it: it
removes the mnemonic mark, trims, lower-cases and folds accents (`"Vídeos"` →
`"videos"`).

## View modifiers

`modifiers::ViewModifiers` is a trait in the prelude. Its methods return the
same element, like SwiftUI's view modifiers. It is implemented for every
type with Freya's `EventHandlersExt`: `rect`, `label`, `Text`, `VStack`,
`HStack`, the kit's view types, such as `Divider`, and every control
([modifiers on controls](05-controls.md#modifiers-on-controls)).

| Method | SwiftUI | What it adds |
|---|---|---|
| `.context_menu(items, on_activate)` | `.contextMenu` | A menu at the pointer on secondary click |
| `.help(text)` | `.help(_:)` | A tooltip |
| `.popover(is_presented, size, content)` | `.popover(isPresented:content:)` | A [`Popover`](#popover) that points at the element while the `State<bool>` is `true` ([11](11-views.md#sheet)) |
| `.focus_ring(corner_radius, color)` | — | A ring around the element |
| `.foreground_style(ShapeStyle)` | `.foregroundStyle(_:)` | The text color of `Text`, `label` or `paragraph` |
| `.background_style(ShapeStyle)` | `.background(_:ignoresSafeAreaEdges:)` | A background paint, with blur for a material |
| `.shadows(Vec<ShadowStyle>)` | `.shadow(color:radius:x:y:)` | Shadows |

The last three take the paint types of the styling system and are described
in [Styles and themes](../../agents/styles.md#paint-on-views).

A modifier sets Freya event handlers on the element: `.context_menu` sets
`on_secondary_down`; `.help` sets `on_pointer_enter`, `on_pointer_move`,
`on_pointer_leave` and `on_pointer_down`; `.popover` sets `on_sized`. A handler for the same event, set
before or after, replaces the other one. On a control the handlers sit on a node
around it, so they never replace the control's own.

### `.context_menu`

`element.context_menu(items, on_activate)` adds a secondary-click handler to
`element`. On a secondary press it opens a `MenuView` at the
pointer, with the popover metrics, as a popup with a grab. Choosing an action
calls `on_activate(id)` and closes the menu; a click outside closes it. The
popup flips and slides to stay on screen.

Text fields and `TextEditor` already have their own edit menu
([6. Text and editing](06-text-and-editing.md)).

### `pop_up_menu`

`popup_button::pop_up_menu(items, rect, side, on_activate, on_close)` opens a
`MenuView` beside `rect`, in surface coordinates, toward `side` (a
`PopupAnchor`: `Top`, `Bottom`, `Left` or `Right`). It is AppKit's
`NSMenu.popUp(positioning:at:in:)`. `on_close` runs when the menu closes,
chosen or dismissed. Malaysia's Dock opens an icon's menu this way, above the
icon on a bottom Dock and beside it on a side Dock, and stays up until
`on_close`.

It is drawn as the Dock menu of macOS 26:

1. The menu is a popover with the `Menu` shape. Its arrow sits in the corner
   where the menu starts, `DOCK_MENU_ARROW_INSET` (30) from the edge, and
   points at the centre of the edge of `rect` that faces the menu.
2. The menu grows to the right of that point, or down beside a side Dock.
   Without room it flips: the arrow moves to the other corner.
3. The popup adds the arrow band, `POPOVER.arrow_height` (13), on the edge
   that faces `rect` ([`Popover`](#popover) has the protocol).
4. The rows take the Dock menu metrics: `DOCK_MENU_PADDING` (5) above and
   below, text and separators `DOCK_MENU_PADDING` + `DOCK_MENU_ITEM_PADDING_X`
   (17) from the edges, separators 12 tall (`DOCK_MENU_SEPARATOR_MARGIN`) and
   a width of at least `DOCK_MENU_MIN_WIDTH` (228).
5. The menu is clear, `theme::over_glass`: the compositor draws the menu glass
   (`Resolved::menu_glass`) under it, arrow included, so the menu takes its
   measured blur. The glass has one veil, lift and cap for the
   light scheme and one for the dark ones. The ink follows the panel, not the
   colour scheme ([Glass over a backdrop](04-appearance-and-tokens.md#glass-over-a-backdrop)).
   With Increase Contrast it keeps its solid fill and takes the contrast inks
   and the strong separator of the contrast palette. Reduce Transparency alone
   leaves the glass, as in macOS.
   A search field in a menu over the glass takes the faint ink of disabled text
   (`glass_disabled_fg`) as a translucent fill, not the grey of `search_bg`, so
   it stays a field on the glass.

### `.help` (tooltips)

`element.help(text)` shows `text` as a tooltip over `element`. `text` is
`impl Into<Cow<'static, str>>`. `.help` calls a hook: call it on
every render, never inside a condition. The tooltip works like this:

1. The pointer enters the element and stays `TOOLTIP.delay_ms` (1500 ms).
2. The tooltip opens as a popup without a grab, `TOOLTIP.pointer_gap` (19 px)
   below the pointer.
3. It closes when the pointer leaves, when a button is pressed and when the
   component is dropped.

It draws `SUBHEADLINE` text on a `TOOLTIP.padding_x` (6) by `padding_y` (3)
padding, with a corner of `TOOLTIP.radius` and the fill and ink of the
appearance (`bg_*`, `ink_*`, opaque with Increase Contrast). A text wider than
`TOOLTIP.wrap_after` wraps at `wrap_width`. It draws no border: the compositor
adds the edge and the shadow. The numbers are in
[the metrics](../research/macos-control-metrics.md#tooltips).

### `.focus_ring`

`element.focus_ring(corner_radius, color)` draws a ring of `FOCUS_RING.width`
(3) in `color` just outside the element: an outer border that follows the
element's corners, or `corner_radius` when the element has none. It needs an
element with a style: `rect`, `VStack`, `HStack` or a rect-like view type, not
`label` or `Text`. A control uses it as a selection
mark; the keyboard focus ring is `Interaction::focus_ring`
([5. Controls](05-controls.md#building-a-new-control)).

The keyboard focus ring draws `Resolved::focus_ring`: as macOS does, a
translucent ring (the `focus_ring` of the accent at 49.8%). Over a light or
busy surface that ring can fall under the 3:1 contrast of WCAG 2.2 §1.4.11. With
Increase Contrast (`contrast`) it is opaque, and a color macOS has no name for
moves until it reaches 3:1 on the window
([4. Accents](04-appearance-and-tokens.md#accents)). The `color` of this
modifier is the caller's and is not changed.

## Popovers

### `Popover`

`popover::Popover` is `NSPopover`. `Popover::new(size, content)` builds it and
`.show(anchor)` opens it below `anchor`, an `Area` in window coordinates, and
returns its `PopupId`. `.on_dismissed(f)` runs `f` when it closes. Use it
directly only for an anchor of your own; `.popover` above tracks the view's
area for you:

| Argument | What it is |
|---|---|
| `size` | `(width, height)` of the content, which is the whole card, as in `NSPopover`: its `contentSize` is the card (200 × 120 content gives a 200 × 120 card with a 13 high arrow). On Linux the popup adds the arrow band, `POPOVER.arrow_height` (13), on top; on macOS AppKit draws the arrow outside the content. Pad the content in the closure |
| `content` | A closure that returns the content as an `Element`. The popover draws the background |
| `anchor` | The area the arrow points at |

It opens with a grab: a click outside closes it, and on macOS Escape too. It
slides sideways to stay on screen.

On macOS it is a real `NSPopover`, transient as SwiftUI's `.popover`, whose
content view is the kit's Metal layer drawn clear: AppKit draws the Liquid
Glass body, the arrow, the shadow and the placement, and follows Reduce
Transparency. A click in a popup opened from the popover keeps it open.

On Linux the app asks Malaysia, with `infinity_window_style_v1.set_popover` (version
2), for the shape of `NSPopover` on macOS 26: a continuous 20 px corner and a
25.42 × 13 px arrow with a round tip, filleted into the body, pointing at the
centre of the anchor. The compositor clips, borders and shadows that shape
([Malaysia ADR 0059](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisions.md)).
The measurements come from the Flutter package
[`popover`](https://pub.dev/packages/popover), taken from AppKit on macOS
26.7. Without version 2, the popup is a plain rectangle.

### Other popover shapes

`Popups::open_popover(spec, content, shape)` opens any popup as a popover.
The compositor puts the arrow on the edge that faces the anchor rectangle:
the bottom edge when the popup opens above it, the right edge when it opens
to its left, the left edge when it opens to its right, and the top edge
otherwise. `popup_button::arrow_band(side, height)` gives that band as `Gaps`
for a popup that opens toward `side`. Add it to `spec.size` and leave it
empty in `content`.

`PopoverShape` picks the corner and the arrow:

| Shape | Corner | Arrow | Used by |
|---|---|---|---|
| `Popover` | `POPOVER.radius` (20) | `POPOVER`, 25.42 × 13 | `Popover` |
| `Menu` | the menu corner, `components.shell.menu.panel.radius` (12; `CORNER_MENU.fitted` 12.5) | `POPOVER`, 25.42 × 13, with the `DOCK_MENU_ARROW_*` tip (6) and fillet (8) | [`pop_up_menu`](#pop_up_menu) |
| `Label` | `DOCK_LABEL_RADIUS` (12, the menu bar's title pill), with circular arcs | `DOCK_LABEL_ARROW_*`, 22 × 6.5 | the name over a Dock icon |

`Menu` and `Label` need `infinity_window_style_v1.set_popover_shape`
(version 7). Without it, the popup keeps a plain frame.

On macOS the runner draws `Menu` and `Label` itself
(`platform/macos/arrow.rs`): the body and the arrow are one outline of
circular arcs, which is the `maskImage` of the popup's `NSVisualEffectView`,
so the material and the window shadow follow it, and a mask of the content
layer. The edge and the tip follow the compositor's rules above. A `Popover`
is an `NSPopover` ([`Popover`](#popover)).

## Helpers

| Function | What it does |
|---|---|
| `normalize_separators(&mut items)` | Removes leading, trailing and repeated separators, also in submenus |
| `measure_text(text, size)` | Width in pixels of `text` in the system sans font at `size` |
| `measure_text_weight(text, size, weight)` | The same with a `FontWeight` |
| `measure_wrapped(text, size, weight, width)` | The widest line and the number of lines of `text` wrapped at `width`: `(f32, usize)` |

The type aliases `ItemsSource`, `SubmenuSource`, `SearchableSource` and
`RevealRoots` name the `Rc` closures that the `with_*` methods above store.

Next: [8. Windows](08-windows.md).
