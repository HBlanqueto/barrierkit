# 5. Controls

**Goal:** add any interactive control to a window and keep its value in the app's state.
**Scope:** the kit's controls, from buttons to the canvas, and how to build a new one. Text
editing is in [6](06-text-and-editing.md); menus, popovers, tooltips and context menus are in
[7](07-menus-and-popovers.md); colors and tokens are in [4](04-appearance-and-tokens.md).
**Reader:** anyone who builds the interface of an app.
**Input:** a window that runs ([2](02-first-app.md)). **Output:** a window with controls whose
values live in `State`.

## Steps

1. Create the value in the view, with a hook at the top of the closure:

   ```rust
   let wifi = use_state(|| true);
   ```

2. Build the control. The first argument of `new` is its accessible label; the next one is
   the `State` it edits:

   ```rust
   Switch::new("Wi-Fi", wifi)
   ```

   Visible text is `impl Into<Cow<'static, str>>`: a literal or `tr!("id")` is
   borrowed, a `String` is owned. No text needs `i18n::intern`.

3. Add it to the view: `VStack::new().child(..)`. When the user flips the switch, it writes
   `wifi`; when `wifi` changes, the view runs again and the switch shows the new value.

Text fields work the same way: they take the `State<String>`
([Text fields](#text-fields)).

## The controlled pattern

| Rule | What happens |
|---|---|
| The app owns the value | It creates the `State` and passes it in `new` |
| The user changes it | The control writes the new value to the `State` |
| The app changes the value | The control shows the app's value |
| The new value equals the old one | No write |

✗ Reading the control's value: there is no getter.
✓ Reading the `State` the control writes.

A callback remains only for actions: `Button::on_press`, `TextField::on_submit`,
`MenuButton::on_activate`. To react to a change, read the `State` in `use_side_effect`.

## Common rules

- The label of `new` is the accessible name. Controls with visible text (`Button`,
  `Checkbox`, `RadioGroup` options) also show it. The text is owned or borrowed, never
  interned, so a list built at run time does not keep each text for the life of the process.
- Tab moves the focus. A keyboard focus draws the focus ring
  (`FOCUS_RING`, 3 px, a translucent accent; opaque with Increase Contrast) only while the
  window is active.
- Space or Enter press the focused button. The arrow keys move sliders, steppers,
  segments and radio options.
- `.disabled(true)` dims the control and stops its callbacks. On any view it does the same to every control inside
  ([16](16-swiftui-to-the-kit.md#disabled)).
- Controls with a fill take `.tint(Tint)` (`bk::backdrop::Tint`: `Accent`,
  `Color(Rgba)`, `Backdrop`; see [15](15-backdrops-and-media.md)).
- Visible text in a real app comes from `tr!` ([17](17-languages.md)); examples here use
  literals.
- Most controls take `.key(..)` (Freya's `KeyExt`) to keep their state in lists.

## Modifiers on controls

Every control takes Freya's container modifiers and the kit's
[`ViewModifiers`](07-menus-and-popovers.md#view-modifiers), in any order. They set
properties on the control's own node, as they do on `VStack`; they do not wrap it, as SwiftUI's
do. The padding is always inside the background; outer space is `.margin`.

| SwiftUI | Kit | Freya trait |
|---|---|---|
| `.padding(_:)` | `.padding(Gaps)`, inside the control | `ContainerExt` |
| `.frame(width:height:)`, `.frame(minWidth:…)` | `.width`, `.height`, `.min_width`, `.max_height`… with a `Size` | `ContainerSizeExt`, `ContainerConstraintsExt` |
| an outer `.padding(_:)` | `.margin(Gaps)`, `.position(Position)` | `ContainerPositionExt` |
| `.background(_:)`, `.border(_:)`, `.cornerRadius(_:)`, `.shadow(…)` | `.background(Fill)`, `.border`, `.corner_radius`, `.shadow` | `BackgroundExt`, `DecorationExt` |
| `.opacity(_:)`, `.scaleEffect(_:)`, `.rotationEffect(_:)`, `.clipShape(_:)` | `.opacity`, `.scale`, `.rotation`, `.overflow` | `EffectExt` |
| `.help(_:)`, `.contextMenu(…)` | `.help`, `.context_menu`, any `.on_*` handler | `ViewModifiers`, `EventHandlersExt` |

✗ A `rect()` that only carries the modifiers of a control:

```rust
rect().margin(SPACE_2).help(tr!("save")).child(Button::new(tr!("save")))
```

✓ The modifiers on the control:

```rust
Button::new(tr!("save")).margin(SPACE_2).help(tr!("save"))
```

- A value you set replaces the control's own: `.padding((0., 20.))` on a `Button` swaps its side
  padding for 20. A value equal to Freya's default (zero padding, `Size::Inner`, a transparent
  background) leaves the control's own: change a control's look with `.style(..)`.
- `.border` and `.shadow` add to the control's own. The last `.padding` or `.background`
  wins.
- A control's own method of the same name wins: `Button::width` sets the button's width.
- A handler (`.help`, `.context_menu`, `.on_*`) sits on a node around the control, so it never
  replaces the control's own handlers. Two handlers for the same event, such as `.help` and
  `.context_menu`, replace each other.
- Not offered: `.direction`, `.spacing` and the other content methods (a control arranges its
  own content) and `.foreground_style` (nothing inherits).
- `StyleScope` (a scope), `Sheet` and `Alert` (modal windows), `SidebarToggle` (a unit struct)
  and `Titlebar` (placed by the window) take none: put the modifiers on a `rect()` that holds
  them.
- `.keyboard_shortcut(Shortcut)` on any view gives it to every `Button` inside that has none
  ([16](16-swiftui-to-the-kit.md#actions)): it reuses the environment scope and the button's key handler.

The program is [`control_modifiers.rs`](../../agents/recipes/control_modifiers.rs).

## Gestures

`.gesture(g)`, `.simultaneous_gesture(g)` and `.high_priority_gesture(g)` of `ViewModifiers`
attach a gesture to any view; the gestures are in `bk::gesture` (recipe 33 of [agents](../../agents/recipes.md)).
A touch pan, a long press and drag, and a pinch need recognizers and a rule for who wins a pointer.

| Gesture | Value | Default |
|---|---|---|
| `TapGesture::new().count(n)` | `TapValue { location }` | 1 tap, 4 points of movement, 330 ms from a release to the next press, any distance between the taps |
| `LongPressGesture::new()` | `LongPressValue { location }`, once the time has passed | 500 ms (`.minimum_duration(ms)`), 10 points (`.maximum_distance(points)`) |
| `DragGesture::new()` | `DragValue { start_location, location, translation, velocity }`, velocity in points per millisecond | 10 points (`.minimum_distance(points)`) |
| `MagnifyGesture::new()` | `MagnifyValue { magnification }`, from two fingers | 0.01 |
| `RotateGesture::new()` | `RotateValue { rotation }` in radians, from two fingers | 1 degree |
| `a.sequenced_before(b)` | `SequenceValue::First(a)`, then `Second(a, None)` once `a` has ended, then `Second(a, Some(b))`; `b` starts from the press that is still down (after a long press) or from the next press (after a tap) | |

Each takes `.on_changed(f)` (when it begins and as it moves) and `.on_ended(f)`.
`.updating(state, |value, state| ..)` is `@GestureState`: `use_gesture_state(initial)` gives a
`GestureState` that follows the gesture and goes back to `initial` when it ends, fails or is
cancelled. The defaults of `TapGesture`, `LongPressGesture` and `DragGesture` are measured on SwiftUI on this Mac ([U6 design](../research/uikit-roadmap.md#u6-design-the-gesture-layer)), in `ctdesign::GESTURE`; the two-finger ones are SwiftUI's documented values.

- A view keeps one handler per event, so two `.gesture` calls on one view replace each other.
  Put the gestures together: `a.exclusively(b)` tries `a` first (while it can still begin,
  `b` waits, as a single tap waits for a double tap; when it begins, `b` fails) and
  `a.simultaneously(b)` lets both run.
- Across views the deepest wins a pointer; a `.high_priority_gesture` beats the gestures of the
  views below it, and a `.simultaneous_gesture` neither wins nor loses. A gesture that begins
  fails the others on its pointer, so a tap that moved is no tap and a drag cancels a long press.
- A mouse and a finger are both pointers; `MagnifyGesture` and `RotateGesture` take two fingers,
  and on macOS a trackpad pinch or rotation (`magnifyWithEvent:`, `rotateWithEvent:`) reaches the
  ones under the pointer as two fingers `gesture::trackpad` stands for (the Wayland runner has no
  source yet: it needs `pointer-gestures-unstable-v1`). Hover is not a gesture: `.on_hover`.
- A finger pan on a `ScrollView` is a `DragGesture::touches_only()` on its content, so a gesture
  inside it wins the finger until the pan has moved 10 points; the release velocity feeds the same
  glide as the wheel and the stretch uses a band of 0.55 (UIKit's documented constant for touch, not
  measured: the Mac probe cannot post a finger). A mouse never pans. `Table` keeps Freya's own pan.
  UIKit delays a touch to a child by 150 ms (`delaysContentTouches`); the kit does not (not measured).

## Buttons

`controls::Button` is a push button with a text label, and a symbol if you
give one.

| Method | Default | Effect |
|---|---|---|
| `Button::new(label)` | — | `label` is visible text, see [step 2](#steps) |
| `Button::system_image(label, Symbol)` | — | A symbol before the label, in the label's color |
| `.label(\|\| view)` | the drawn text | A closure that returns the view drawn in place of the text; the string of `new` stays the accessible name |
| `.style(\|s\| ..)` | Tengen, SwiftUI's `.bordered` | Look: a preset or any field of `ButtonStyle`, see below |
| `.size(ControlSize)` | `Regular` | Height, padding, font and corners; see [control size](#control-size) |
| `.width(Size)` | `Size::auto()` | Fixed or fill width |
| `.toggled(bool)` | not a toggle | Makes it a toggle; `true` takes the `bordered_prominent` look and reports `toggled` |
| `.disabled(bool)` | `false` | Dims it, no `on_press` |
| `.keyboard_shortcut(Shortcut)` | none | Calls `on_press` on that key while the button is enabled; a menu item with the same key wins |
| `.role(ButtonRole)` | none | `Destructive` draws `ButtonStyle::destructive` |
| `.on_press(\|_\| ..)` | — | Called on click, Space or Enter |

The closure runs inside the button, on every render, so state it reads keeps the label current.
`Text`, `Label` and Freya's `label()` take the button's font and the ink of its current state:
pressed, disabled, the `on_accent` ink of `bordered_prominent`, an unfocused window. A bare
`Image::system` keeps its own color; use `Label`. `Button` scales its text and its symbol with the
text size ([chapter 4](04-appearance-and-tokens.md#the-appearance-file)), as `Text` does: the button
gets wider and keeps its height, and a `Text` or `Label` in the closure is pixel equal to the drawn
label at every size. A Freya `label()` takes the family, size, weight and ink but not the line
height, so it is pixel equal only at the default size (checked at 1.3); use `Text`.

```rust
// ✗ the symbol keeps the label color on the accent fill
Button::new("Add").style(ButtonStyle::bordered_prominent).label(|| Image::system(Symbol::Plus))
// ✓
Button::new("Add").style(ButtonStyle::bordered_prominent).label(|| Label::new("Add", Symbol::Plus))
```

The program is [`button_label.rs`](../../agents/recipes/button_label.rs).

Presets of `ButtonStyle`, given as `.style(ButtonStyle::glass)`:

| Preset | Look |
|---|---|
| none (Tengen) | Control background; the pressed fill while held |
| `bordered_prominent` | Accent fill, `on_accent` text |
| `glass` | Capsule with a hairline border, raised background; on macOS 26 and later a native glass in the bordered shape (a capsule from `Large`), the label at full strength and a veil under the pointer |
| `glass_prominent` | Capsule with a hairline border and the accent fill; on macOS 26 and later a native glass tinted with the accent |
| `borderless` | No background until pressed |
| `circular` | A 17 high circle for an icon-only button (AppKit round) |
| `small_square` | A 21 high rectangle with a light gradient and a 1 wide border (`NSButton.BezelStyle.smallSquare`) |
| `textured_rounded` | A 20 high capsule with a faint ring and no fill; the toolbar bezel (`texturedRounded`, `toolbar`) |
| `textured_square` | A 20 high rounded rectangle, nearly white in light and 5% white in dark (`texturedSquare`) |
| `inline` | An 18 high capsule with a 5% fill and a 11 semibold secondary label (`inline`) |
| `plain` | Only the label, as high as a line of text, in the label color (SwiftUI `.plain`) |
| `link` | `plain` with the accent color, darker while pressed (SwiftUI `.link`) |
| `push_disclosure` | A 24 square with the push fill, for a chevron symbol (`pushDisclosure`, `roundedDisclosure`) |

SwiftUI names: `.automatic` and `.bordered` are `ButtonStyle::tengen()`, `.borderedProminent` is
`bordered_prominent`, `.plain` and `.link` are `plain` and `link`, `.glass` and `.glassProminent` are
`glass` and `glass_prominent`, and `Button(role: .destructive)` is `destructive`. SwiftUI's
`.borderless` draws only the label (41 × 16 at regular); the kit's `borderless` keeps the 24 high
hit area, so use `plain` or `link` for the label-only look. `.buttonBorderShape(.capsule)` is
`Shape::Capsule` in the closure of `.style`, and `.circle` is `circular`. A `Toggle` is a `Switch`,
a `Checkbox` or a `Button` with `.toggled(bool)`; a `Picker` is a `SegmentedControl` (`.segmented`),
a `PopUpButton` (`.menu`) or a `RadioGroup` (`.radioGroup`, `.inline`); `.palette` is `ColorWell`.

`ButtonStyle::tengen()` is also what AppKit draws for the `flexiblePush`, `roundRect`,
`recessed`, `accessoryBar` and `accessoryBarAction` bezels: the bench finds them identical to the
push button. `HelpLink` is the `helpButton` bezel and `circular` the `circular` one. A pressed
`small_square`, `textured_rounded` or `textured_square` takes the fill of `BezelPalette`; a
disabled `small_square` is half as opaque; the others change only their label.

A pressed button takes the pressed fill and does not scale (`BUTTON.pressed_scale` is 1).

```rust
Button::new("Cancel")
Button::new("Save").style(ButtonStyle::bordered_prominent)
Button::new("Delete").disabled(true)
```

### Style

`Button` draws only from a `ButtonStyle`. `.style(..)` takes a preset or a
closure for one button, `.control_style(|s: ButtonStyle| ..)` for every button
inside a view and `Theme::tengen().style(ButtonStyle::glass)` for the whole
app. The fields, the presets and `.size` are in
[Styles and themes](../../crates/bk/src/styles/button.rs).

```rust
Button::new("Save").style(|s| ButtonStyle { shape: Shape::Capsule, ..s.bordered_prominent() })
```

## Control size

`.control_size(ControlSize)` on a view sizes every control inside it, as SwiftUI's
`.controlSize(_:)` does, and `.size(ControlSize)` on one control picks its size and wins over it.
Both take one of five sizes on `Button`, `PopUpButton` (also with
`pulls_down`), `SegmentedControl`, `Checkbox`, `RadioGroup`, `Switch`, `Slider`,
`Stepper`, `TextField`, `SearchField`, `ComboBox`, `DatePicker` and `TabView`; `.control_size` also sizes
`ProgressView` and `Spinner`. `Regular` is the default. The tokens of each size are in `ctdesign`
(`BUTTON.size(size)`, `CHECKBOX.size(size)`, `SWITCH.size(size)`, `SLIDER.size(size)`,
`SEGMENTED.size(size)`, `STEPPER.size(size)`), measured on macOS 27.2.1 in
[`macos-control-metrics.md`](../research/macos-control-metrics.md#control-sizes).

| `ControlSize` | Button, pop-up, segmented | Button padding | Font | Corner | Checkbox, radio | Switch | Slider | Stepper |
|---|---|---|---|---|---|---|---|---|
| `Mini` | 16 | 8.6 | 9 | 4 | 12 | 36 × 16 | 12 | 13 × 16 |
| `Small` | 20 | 10 | 11 | 5 | 14 | 44 × 20 | 14 | 17 × 20 |
| `Regular` | 24 | 12 | 13 | 6 | 16 | 54 × 24 | 16 | 20 × 24 |
| `Large` | 28 | 14 | 13 | capsule | 18 | 64 × 28 | 20 | 23 × 28 |
| `ExtraLarge` | 36 | 18 | 13 | capsule | 18 | 80 × 36 | 20 | 30 × 36 |

```rust
Button::new("Save").size(ControlSize::Large)
Checkbox::new("Remember", remember).size(ControlSize::Small)
Slider::new("Volume", volume).size(ControlSize::Mini)
VStack::new().child(Button::new("Save")).child(Slider::new("Volume", volume)).control_size(ControlSize::Small)
```

`.control_size` is a container-level style override ([styles](../../agents/styles.md#levels)): it
changes the style of each sized control inside the view, so a theme comes before it and an
instance `.size(..)` or `.style(..)` after it. A circular button keeps its own size.

| `ControlSize` | Text field | Search field, combo box | Field font |
|---|---|---|---|
| `Mini` | 19 | 16 | 9 |
| `Small` | 22 | 20 | 11 |
| `Regular` | 24 | 24 | 13 |
| `Large` | 24 | 28 | 13 |
| `ExtraLarge` | 24 | 36 | 13 |

The label and the glyphs follow the size: a `Mini` pop-up has a 9 label and 9 high
arrows, and a checkmark is as large as its box. A search field moves its magnifier and clear
button, and a combo box its arrow button and chevron, with the size (`SEARCH_FIELD`,
`COMBO_BOX`).

## Compact buttons

Module `compact`.

| Item | Build | Reports |
|---|---|---|
| `IconButton` | `IconButton::new(Symbol, label)`, `.disabled(bool)`, `.style(\|s\| ..)` | `.on_press(\|_\| ..)` |
| `Stepper` | `Stepper::new(label, State<i64>, min, max)` (shown clamped), `.disabled(bool)`, `.style(\|s\| ..)` | — (writes the `State`) |
| `ToggleGroup` | `ToggleGroup::new(label)`, `.item(text, Face, label, State<bool>)` per toggle, `.style(\|s\| ..)` | — (each toggle writes its `State`) |

- `IconButton` is 30 × 30 with a 16 symbol (`ICON_BUTTON`). Inside a
  [`ToolGroup`](#toolbar-controls) or a `Titlebar` it becomes a capsule as tall as the bar's
  platter, 38 × 38 in the unified bar, 26 × 24 in the compact one and 30 × 28 in the expanded one
  (`TOOLGROUP`), and its
  symbol grows to 21 with a 1.3 stroke (`ICON_BUTTON.toolbar_icon`, `toolbar_stroke`): the size
  AppKit draws toolbar symbols at ([sizes](../research/macos-control-metrics.md#toolbar-items-and-navigation-buttons)).
  In an inactive window a toolbar symbol takes `Role(TertiaryLabel)`, as AppKit dims toolbar
  items (Dark, measured 2026-11-23: ink 0.29 over the platter, `tertiaryLabelColor` is 0.25).
  Under the pointer it takes `Role(Label)` and the hover circle, as AppKit lights the item even
  in an inactive window; the window takes the pointer while it is not key. On native glass the
  circle is 32 wide (`TOOLBAR_ITEM_HIGHLIGHT`); an item alone in its `ToolGroup`, as the sidebar
  toggle, is lit over its whole capsule, and stronger in dark (`alone_hover_dark`), as SwiftUI's.
- `Stepper` shows the value and, 8 to its right, the arrows: 20 × 24, up above down
  (`STEPPER`). ↑ and ↓ also step. An arrow is dimmed at its limit.
- `ToggleGroup` is a row of independent toggles on one background, for text formatting.
  `Face` (`Bold`, `Italic`, `Underline`) sets how `text` is drawn. A toggle that is on takes
  the accent.

```rust
let cores = use_state(|| 4);
let (bold, italic, underline) = (use_state(|| true), use_state(|| false), use_state(|| false));
IconButton::new(Symbol::Plus, "Add")
Stepper::new("Cores", cores, 1, 16)
ToggleGroup::new("Format")
    .item("B", Face::Bold, "Bold", bold)
    .item("I", Face::Italic, "Italic", italic)
    .item("U", Face::Underline, "Underline", underline)
```

### Style

`IconButton` draws only from an `IconButtonStyle`, `Stepper` from a
`StepperStyle` and `ToggleGroup` from a `ToggleGroupStyle`. The arrows of a
[`DatePicker`](#date-picker) draw from the `StepperStyle` held by its `DatePickerStyle`, 13 wide and as high as the field. The
fields are in [Styles and themes](../../crates/bk/src/styles/icon_button.rs).

```rust
Stepper::new("Cores", cores, 1, 16).style(|s| StepperStyle { value_position: Some(Edge::Trailing), ..s })
```

## Menu buttons

Module `buttons`. The menus they open are [`MenuItem`](07-menus-and-popovers.md) lists, shown
as a popover menu below the button.

| Item | Build | Reports |
|---|---|---|
| `MenuButton` | `MenuButton::new(label, items)`, `MenuButton::system_image(label, Symbol, items)` (only the symbol, borderless, like the «⋯» of a list row, lit under the pointer like `PopUpButton`; `label` names it for screen readers), `.label_style(LabelStyle::TitleAndIcon)` (the symbol before the label, on the bezel), `.style(\|s\| ..)` | `.on_activate(\|id: String\| ..)` |
| `ComboButton` | `ComboButton::new(label, items)`, `.style(\|s\| ..)` | `.on_press(\|_\| ..)` for the label, `.on_activate(\|id: String\| ..)` for the menu |
| `HelpLink` | `HelpLink::new()`, `.style(\|s\| ..)` | `.on_press(\|_\| ..)` |
| `ControlGroup` | `ControlGroup::new(label)`, `.child(..)`, `.style(\|s\| ..)` | — (its children report) |
| `ShareLink` | `ShareLink::new(label, content)`, `.style(\|s\| ..)` | — (acts by itself) |

- `MenuButton` is a button with a down chevron that opens a menu of actions.
- `ComboButton` has two halves split by a thin line: the label runs the main action, the
  chevron opens the menu.
- `HelpLink` is a 24 round button with "?".
- `ControlGroup` joins its children on one background with separators. Give child buttons
  the `borderless` preset.
- `ShareLink` opens a menu with Copy, Send by Email and, when `content` starts with
  `http://` or `https://`, Open With. Copy uses the clipboard; Send by Email opens
  `mail_uri(content)` and Open With opens the link, both through the `OpenURI` portal
  ([13](13-app.md)). freedesktop has no share
  portal, so these are the actions.
- `mail_uri(content)` returns `mailto:?body=…` with every byte but unreserved characters
  percent-encoded.

### Style

`MenuButton` draws only from a `MenuButtonStyle`, `ComboButton` from a
`ComboButtonStyle`, `HelpLink` from a `HelpLinkStyle` and `ControlGroup` from a
`ControlGroupStyle`, which paints only the group and its separators: its
children keep their own styles. `ShareLink` is a `MenuButton`, so it takes a
`MenuButtonStyle`. The fields are in
[Styles and themes](../../crates/bk/src/styles/menu_button.rs).

```rust
MenuButton::new("Sort", items).style(|s| MenuButtonStyle { shape: Shape::Capsule, ..s })
```

## Toolbar controls

| Item | Module | Build | Behavior |
|---|---|---|---|
| `ToolGroup` | `controls` | `ToolGroup::new()`, `.child(..)`, `.children(..)`, `.style(\|s\| ..)` | Puts each child in a 34.5 × 38 slot (36 × 24 in the compact bar, 38 × 28 in the expanded one) on a capsule background; only the compact bar draws separators, 1 × 14 |
| `ToolbarSearch` | `compact` | `ToolbarSearch::new(value, placeholder)`, `.collapsed(bool)`, `.style(\|s\| ..)` | A 180 wide `SearchField` (`TOOLBAR_SEARCH_WIDTH`). Collapsed, it shows a magnifying-glass `IconButton` until pressed or while the text is not empty |
| `ToolbarOverflow` | `compact` | `ToolbarOverflow::new(items)`, `.on_activate(\|id: String\| ..)`, `.style(\|s\| ..)` | A `»` `IconButton` that opens the items as a menu |

Where they sit in the title bar is in [9](09-titlebars.md).

On macOS 26 and later a `SegmentedControl`, the `ToolbarSearch` field and a
`Button` in the toolbar each sit on their own native glass capsule, 36 high, as
SwiftUI's ([15](15-backdrops-and-media.md#on-macos)). The segmented control
draws veil thumbs and no dividers, the search field only its ink, and a button
with an image shows the image alone in a 36 circle; `bordered_prominent` tints
that glass with the accent. Collapsed, `ToolbarSearch`'s button sits in a
`ToolGroup`.

### Style

`ToolGroup` draws only from a `ToolGroupStyle`: slot size, separators,
background, rim and `axis` (a vertical group stacks its items). Its Tengen
takes the opaque colors with Reduce Transparency. The fields are in
[Styles and themes](../../crates/bk/src/styles/tool_group.rs).

`ToolbarSearch` takes a `ToolbarSearchStyle` with only its `width`: the field
draws from the `SearchFieldStyle` and the collapsed button from the
`IconButtonStyle` in context. `ToolbarOverflow` is an `IconButton`, so its
`.style` takes an `IconButtonStyle`.

```rust
ToolGroup::new().child(back).child(forward).style(|s| ToolGroupStyle { axis: Axis::Vertical, ..s })
```

## Switch and checkbox

| Item | Build |
|---|---|
| `Switch` | `Switch::new(label, State<bool>)` |
| `Checkbox` | `Checkbox::new(label, State<bool>)`, `.width(Size)`, `.mixed(bool)` (a dash instead of the check) |

Both take `.disabled(bool)`, `.tint(Tint)` and `.style(|s| ..)`.

- `Switch` is 54 × 24 with a 32 × 20 knob that slides over `SWITCH.transition_ms`. Its label is
  only accessible; put the visible one in a [form row](11-views.md) or set the style's
  `label_position`.
- Hold the knob to drag it. It follows the cursor, grows to `SWITCH.knob_pressed_scale` (1.65:
  as wide as the track and taller than it) and only lets go on an end: the one it is nearer to.
  The switch writes its `State` as soon as the knob passes the middle of the travel, not on
  release, and again if it goes back. The track stays on while you drag and turns off only when the knob
  reaches the off end; it turns on again as soon as the knob leaves it. A press that moves less
  than `DRAG.threshold` is a click and flips the switch when released, as does Space.
- A held `Switch`, `Slider` or `LevelSlider` knob is Liquid Glass the compositor draws, with no
  code in the app: the control reports the knob's rectangle and the compositor adds the lens, the
  outline, the light line and the shadow, in light and dark. The outline is dark and takes the
  colour of what lies under it. `knob_glass` is only the tint (`None` keeps the opaque knob) and
  the control drops its own knob shadows while the compositor draws the knob. Without the
  compositor the knob stays the opaque `knob` paint: the kit draws no knob glass. Draw no knob by hand.
- `Checkbox` is a 16 box with a check mark (`CHECKBOX`), followed by the label. Space flips it;
  Enter goes to the default button, as in AppKit.

```rust
let (off, on, remember) = (use_state(|| false), use_state(|| true), use_state(|| true));
Switch::new("Off", off)
Switch::new("On", on)
Switch::new("Inactive", on).disabled(true)
Checkbox::new("Remember me", remember)
```

### Style

`Switch` draws only from a `SwitchStyle` and `Checkbox` from a
`CheckboxStyle`; `.tint(Tint)` sets their `fill`. Both styles place the label
with `label_position` (an `Edge`). The fields are in
[Styles and themes](../../crates/bk/src/styles/switch.rs).

```rust
Checkbox::new("Remember me", remember).style(|s| CheckboxStyle { label_position: Some(Edge::Leading), ..s })
```

## Choices

Each takes in `new` the `State` of the chosen index and writes it when the user picks.

| Item | Module | Build |
|---|---|---|
| `RadioGroup` | `choices` | `RadioGroup::new(label, State<usize>)`, `.option(text)`, `.disabled(bool)`, `.tint(Tint)`, `.style(\|s\| ..)` |
| `SegmentedControl` | `controls` | `SegmentedControl::new(label, State<usize>)`, `.segment(text)`, `.disabled(bool)`, `.style(\|s\| ..)` |
| `PopUpButton` | `popup_button` | `PopUpButton::new(label, State<usize>)`, `.pulls_down(bool)` (the first item is the title and the chevron points down; the chosen item is still written to the `State`; for actions use [`MenuButton`](#menu-buttons)), `.item(text)`, `.icon_item(text, MenuIcon)`, `.section_header(title)` (a separator and a title before the next item), `.separator()` (a line before the next item), `.disabled(bool)`, `.style(\|s\| ..)` |
| `ColorWell` | `choices` | `ColorWell::new(label, swatches, State<Option<usize>>)` (`None`: no swatch chosen), `.multicolor(name)`, `.disabled(bool)`, `.style(\|s\| ..)` |
| `ChoiceCards` | `choices` | `ChoiceCards::new(label, State<usize>)`, `.option(caption, preview)`, `.disabled(bool)`, `.style(\|s\| ..)` |

- `RadioGroup` stacks its options in a column; its style's `axis` puts them in a row. The
  arrow keys move the choice and wrap at the ends.
- `SegmentedControl` is as wide as its content: each segment is its text plus 12 on each
  side, the widths of AppKit's `fit` distribution. A wider `.width(..)` shares the extra in
  proportion. Each segment is 24 tall. The chosen one is filled with the toggle accent and
  a 14 high line separates two neighbors that are not chosen. The arrows move the choice and
  wrap.
- `PopUpButton` shows the chosen item, its SVG icon if it has one, and an up-down chevron.
  Under the pointer a rounded background covers all three and the chevron's circle melts
  into it, like macOS. A click opens the items as a menu with a check mark, placed so the
  chosen row covers the button. The menu is clear (`theme::over_glass`): Malaysia's compositor draws the menus'
  glass under it, as under the menu bar's. `MenuIcon` is in [7](07-menus-and-popovers.md).
- `ColorWell` is a row of 24 swatches, 12 apart, measured on the accent picker of
  System Settings. The chosen one gets an accent ring 1 away
  (`SETTINGS.swatch_ring_gap`) and 3 wide, like macOS. The chosen swatch and the one
  under the pointer show their name below. `swatches` is a slice of
  `(name, Rgba)`, for example `&ACCENT_SWATCHES`; a name is a literal, a `tr!` result
  or a `String`, and the well copies it. A borrowed name that is an accent id (`"blue"`…) shows
  its translation, `swatch_name(id)`.
  `ColorWell::accents(label, selection)` is the accent picker: it takes `ACCENT_SWATCHES` and draws
  each swatch in the `control` color of the current appearance, as System Settings does.
  `.multicolor(name)` puts a rainbow swatch first, from `ctdesign::MULTICOLOR`,
  like «Multicolor» in macOS; it is index 0 and the swatches follow.
- `ChoiceCards` shows 66 × 44 previews with a caption below; the chosen card gets an accent
  ring, `selection_gap` away (0 by default). System Settings draws its icon styles on
  `SETTINGS.icon_card_size` cards with the ring `icon_card_ring_gap` away, like macOS.

```rust
let (tint, period, lights) = (use_state(|| 0), use_state(|| 1), use_state(|| 1));
let accent = use_state(|| Some(0));

RadioGroup::new("Apply the tint to", tint)
    .option("Windows only")
    .option("The whole desktop")

SegmentedControl::new("Period", period)
    .segment("Day").segment("Week").segment("Month")

PopUpButton::new("Traffic lights", lights)
    .item("Automatic").item("Classic").item("Accent")

ColorWell::accents("Accent color", accent).multicolor("Multicolor")
```

### Style

`SegmentedControl` draws only from a `SegmentedControlStyle`: the group, each
segment, the chosen one and `axis` (vertical stacks the segments). The fields
are in [Styles and themes](../../crates/bk/src/styles/segmented_control.rs).

`RadioGroup` draws only from a `RadioGroupStyle`, with the `axis` of its
options and the `label_position` of each label; `.tint(Tint)` sets its `fill`.
`ColorWell` draws from a `ColorWellStyle` and `ChoiceCards` from a
`ChoiceCardsStyle`, each with an `axis` and `wraps` for its swatches or cards.
The fields are in [Styles and themes](../../crates/bk/src/styles/radio_group.rs).
`PopUpButton` draws from a [`PopUpButtonStyle`](../../crates/bk/src/styles/popup_button.rs):
background, label, round indicator and chevron.

```rust
SegmentedControl::new("View", view).segment("Icons").segment("List")
    .style(|s| SegmentedControlStyle { axis: Axis::Vertical, ..s })
RadioGroup::new("Size", size).option("Small").option("Large")
    .style(|s| RadioGroupStyle { axis: Axis::Horizontal, ..s })
```

## Sliders

| Item | Module | Build |
|---|---|---|
| `Slider` | `controls` | `Slider::new(label, State<f32>)`, `Slider::stepped(label, steps, State<f32>)` (at least 2 steps), `.range(lo..=hi)`, `.labels(&[..])`, `.number_of_tick_marks(n)`, `.continuous(bool)`, `.disabled(bool)`, `.tint(Tint)`, `.style(\|s\| ..)` |
| `CircularSlider` | `controls` | `CircularSlider::new(label, State<f32>)` (0 to 1, once around from the top), `.disabled(bool)`, `.style(\|s\| ..)` |
| `LevelSlider` | `level` | `LevelSlider::new(label, State<f32>)` (0 to 1), `.disabled(bool)`, `.tint(Tint)`, `.style(\|s\| ..)` |

- `Slider` is 220 wide. `Slider::new` is free: the value is continuous inside `.range(lo..=hi)`
  (0 to 1 by default), and the knob goes to wherever the cursor is, over a mark or between two,
  and stays where you drop it. `.number_of_tick_marks(n)` spreads `n` ticks evenly and
  `.labels(&[..])` puts captions under the track (literals, `tr!` results or `String`s; the
  slider copies them); neither snaps the knob, as `NSSlider` with
  `allowsTickMarkValuesOnly` off. ← ↓ and → ↑ move 5% of the range (`SLIDER_KEY_STEP`).
- `Slider::stepped` is for a choice among a few options. Its value is the step, from 0 to
  `steps - 1`, with a tick under each step and an optional caption per step. A click or drag
  snaps to the nearest step; the arrows move one step. Fewer captions than steps sit at the
  ends and name no step.
- `.continuous(false)` is `NSSlider.isContinuous`: the slider writes its `State` once, with the value where
  you let go, instead of at every move. The knob still moves as you drag. An arrow key still writes at once.
  Use it when each write saves a file or changes the system.
- A new value glides in `DURATION_FAST_MS` (`SliderStyle::animation`). While you drag, the knob
  does not wait: a free one follows the pointer and a stepped one jumps from step to step.
- While you hold a knob it grows by `SLIDER.knob_pressed_scale` (1.3) and turns into Liquid
  Glass the compositor draws ([platform](18-platform.md), `set_knobs`): the control background
  at `SLIDER.knob_glass_alpha` (0.3) over the Dock's lens (`DOCK_LENS_*`) turned outward, which
  refracts the panel past the knob; its outline is part of the glass, as the switch's, with no
  frame lines. It does not
  depend on the Liquid Glass blur setting. `knob_pressed_scale: 1.0` or `knob_glass: None` in the
  style turn it off.
- `CircularSlider` is a 24 disc with a dash from 3 to 9 off the center that points at the value
  (`CIRCULAR_SLIDER`). A drag turns it; the arrows move it by `SLIDER_KEY_STEP`.
- `LevelSlider` fills its width. The arrows move it by `SLIDER_KEY_STEP` (0.05).
- Its Tengen is a 6 track with a 19.5 × 16 knob. `.style(|s| s.fat(Symbol))` is the 22 tall
  Control Center bar with the symbol inside the fill.

```rust
let (brightness, dock, level) = (use_state(|| 0.3), use_state(|| 48.0), use_state(|| 2.0));
let volume = use_state(|| 0.6);
Slider::new("Brightness", brightness)
Slider::new("Dock size", dock).range(16.0..=128.0).labels(&["Small", "Large"])
Slider::stepped("Level", 5, level).labels(&["A", "B", "C", "D", "E"])
LevelSlider::new("Brightness", brightness)
LevelSlider::new("Volume", volume).style(|s| s.fat(Symbol::SpeakerWave))
```

### Style

`Slider` draws only from a `SliderStyle` ([ADR 0064](../decisions.md)).
`.tint(Tint)` sets its `fill`; `.style(|s| ..)` changes any other field for one slider,
`.control_style(|s: SliderStyle| ..)` for every slider inside a view and
`Theme::tengen().style(|s: SliderStyle| ..)`
for the whole app. The levels, the fields and the value types are in
[Styles and themes](../../agents/styles.md). `LevelSlider` draws the same way from a
`LevelSliderStyle` ([fields](../../crates/bk/src/styles/level_slider.rs)) and `CircularSlider`
from a [`CircularSliderStyle`](../../crates/bk/src/styles/circular_slider.rs).

```rust
Slider::new("Volume", volume).style(|s| SliderStyle { track_height: 10.0, ..s })
```

## Text fields

One-line fields write straight into a `State<String>` that the app owns. Their edit menu and
spell checking are in [6](06-text-and-editing.md#edit-menu).

| Item | Module | Build | Options |
|---|---|---|---|
| `TextField` | `controls` | `TextField::new(value, placeholder)` | `.width(Size)` (fill), `.size(ControlSize)`, `.disabled(bool)`, `.auto_focus(bool)`, `.autocorrection_disabled(bool)`, `.style(\|s\| ..)` (`TextFieldStyle::plain` for no bezel), `.on_submit(\|text: String\| ..)` |
| `SearchField` | `controls` | `SearchField::new(value, placeholder)` | `.width`, `.size(ControlSize)`, `.disabled(bool)`, `.auto_focus`, `.style`, `.tokens(State<Vec<String>>)`, `.a11y_id(AccessibilityId)`, `.on_focus_change(\|focused: bool\| ..)`, `.on_navigate(\|key: NamedKey\| ..)` |
| `SecureField` | `controls` | `SecureField::new(State<PasswordField>, label)` | `.width`, `.disabled(bool)`, `.auto_focus`, `.revealed(bool)` (shows the text, for «Show password»), `.style`, `.on_submit(\|secret: Zeroizing<String>\| ..)` |
| `ComboBox` | `combo_box` | `ComboBox::new(value, placeholder)` | `.item(text)`, `.width(Size)`, `.size(ControlSize)`, `.disabled(bool)`, `.style(\|s\| ..)` |

- All are 24 tall with a 6 side padding at `Regular`. `SecureField` draws the focus ring; the
  others turn their border to the accent while focused.
- A disabled field keeps its bezel and turns its text, placeholder and magnifier to the
  tertiary label, like AppKit's; it takes no focus or input. `TextFieldStyle::plain` is
  `.textFieldStyle(.plain)`: no bezel or fill, 16 high, the text 2.25 from the edge.
- `auto_focus(true)` takes the focus when the field mounts.
- `autocorrection_disabled(true)` turns [spell checking](06-text-and-editing.md#spelling) off,
  for text that is not prose, such as a network name.
- `SearchField` shows a clear button while it has text. `on_navigate` receives ↑, ↓ and Enter
  so a result list can follow the keys. `tokens` shows blue rectangles before the text, like the
  filters in Finder; Backspace on an empty field removes the last one.
- `SecureField` shows one 5.75 dot per character, 8 apart. Enter hands the value to `on_submit` and empties
  the field ([Passwords](#passwords)).
- `ComboBox` is a text field with a 24 × 20 arrow button 4 from its right edge. The button or ↓
  opens the items as a menu; picking one writes it into `value`. Free typing also works.

```rust
let name = use_state(|| "jappleseed".to_string());
TextField::new(name, "User name")
SearchField::new(query, "Search").auto_focus(true)
let password = use_state(PasswordField::default);
SecureField::new(password, "Password").on_submit(move |secret| sign_in(secret))
ComboBox::new(shell, "Shell").item("zsh").item("fish")
```

### Style

`TextField`, `SearchField` and `SecureField` draw only from a
`TextFieldStyle`, a `SearchFieldStyle` and a `SecureFieldStyle`: size, shape,
background, border, text, placeholder and font. The fields, and what Freya's
`Input` can draw, are in
[Styles and themes](../../crates/bk/src/styles/text_field.rs).

`ComboBox` draws its field from the `TextFieldStyle` in context and its arrow
button from a [`ComboBoxStyle`](../../crates/bk/src/styles/combo_box.rs).

```rust
TextField::new(name, "User name").style(|s| TextFieldStyle { shape: Shape::Capsule, ..s })
```

## Passwords

`password::PasswordField` holds a secret in a `Zeroizing<String>` and a `password::PasswordState`
(`Idle`, `Verifying`, `Error(Cow<'static, str>)`). `SecureField` uses it; a custom field can too.

| Method | Effect |
|---|---|
| `PasswordField::default()` | Empty, `Idle` |
| `handle_key(&key, modifiers)` | Inserts text, Backspace deletes, Enter returns `take()` |
| `insert(char)`, `backspace()` | Edit; ignored while `Verifying`; an `Error` returns to `Idle` |
| `take()` | Returns the value and moves to `Verifying`; `None` if empty or already verifying |
| `set_error(message)` | Wipes the value and moves to `Error` |
| `reset()` | Wipes the value and moves to `Idle` |
| `state()`, `masked()` | The state and the `•` string |

Growing the buffer copies it into a new zeroized one, so no copy of the secret is freed
without being wiped. `SecureField.revealed(true)` copies the text into a Freya label, which cannot
wipe it.

## Color picker

`color_picker::ColorPicker::new(label, State<Rgba>)` is a 48 × 24 well filled with the color. A click
opens a [popover](07-menus-and-popovers.md) with the hex value and hue, saturation and
brightness `LevelSlider`s. Each move writes the `State`. `.disabled(bool)` draws the whole well
at 50% and takes no click.
`ColorPanel::show(color, area, on_change)` opens the same popover beside `area` without a
well, like `NSColorPanel`, for a menu item such as «Choose Color…».

| Helper | Returns |
|---|---|
| `hsb_to_rgb(hue, saturation, brightness)` | Opaque `Rgba`; all inputs 0 to 1 |
| `rgb_to_hsb(rgba)` | `(hue, saturation, brightness)`, 0 to 1 |
| `hex(rgba)` | `"#RRGGBB"` |

To choose from preset colors, use [`ColorWell`](#choices).

### Style

`ColorPicker` draws from a [`ColorPickerStyle`](../../crates/bk/src/styles/color_picker.rs):
the well's size, shape and border, and the panel's size and captions. The
color itself is content. The default is the capsule bezel with the swatch inside, like
`NSColorWell.Style.default`; two presets take the other styles:

```rust
ColorPicker::new("Color", accent).style(ColorPickerStyle::minimal)
ColorPicker::new("Color", accent).style(ColorPickerStyle::expanded)
```

`minimal` is the capsule swatch alone. `expanded` is a 47 × 24 rounded swatch (radius 5) and a
24 square button with a 16 color wheel (`COLOR_PICKER.wheel_*`), 72 wide in all.

## Date picker

`date_picker::DatePicker::new(label, State<CivilDateTime>)` is a text field with one segment per part
and a stepper beside it, like the `textFieldAndStepper` style of `NSDatePicker`. The
stepper's arrows draw from the [`StepperStyle`](../../crates/bk/src/styles/stepper.rs) of its `DatePickerStyle`.

| Method | Default | Effect |
|---|---|---|
| `.elements(DateElements)` | `Date` | `Date`, `Time` or `DateAndTime` |
| `.field()` | field and stepper | `.datePickerStyle(.field)`: the field alone, 80 × 23 with its border outside, like the `textField` style of `NSDatePicker` |
| `.size(ControlSize)` | `Regular` | The field stays 78 wide and 21 high; the stepper takes 9 × 13, 11 × 16, 13 × 20, 22 × 28 or 29 × 36, and `Large` and `ExtraLarge` make the field 30 and 38 high, its border included (`DATE_PICKER.mini` to `xl`) |
| `.disabled(bool)` | `false` | Tertiary label ink, a disabled stepper and no input |
| `.graphical()` | the field | Shows the 139 × 148 month calendar itself, like the `clockAndCalendar` style of `NSDatePicker`: the month and year, ◀ ● ▶ (the dot goes to today), the weekdays over a line and right-aligned days. The chosen day is gray until the calendar has focus, then the selection color |

| Input | Effect |
|---|---|
| ↑ ↓ or the arrows | Change the chosen part; hour and minute wrap |
| ← → | Move between parts |
| Digits | Type the part; it moves on when full or when no other digit fits (a 4 in the day) |
| Click on a date part | Opens the 139 × 148 month calendar below |

The order of the parts (`date-order`: `dmy`, `mdy` or `ymd`) and the first day of the week
(`week-start`) come from the language file `bk.ftl`.

```rust
let moment = use_state(|| CivilDateTime::new(CivilDate::new(2026, 11, 6), 9, 30));
DatePicker::new("Date", moment)
    .elements(DateElements::DateAndTime)
```

Module `date` does the calendar arithmetic without dependencies, with
[Howard Hinnant's algorithms](https://howardhinnant.github.io/date_algorithms.html).

| Item | What it does |
|---|---|
| `CivilDate { year, month, day }` | `new` clamps the month to 1–12 and the day to the month, as `Calendar` does when a month or year change leaves no such day. `today()` is the local date, or the UTC date when the local time is unavailable. `add_days`, `add_months`, `shifted(part, delta)`, `weekday_from_monday()`, `month_grid(monday_first)` (42 days) |
| `CivilDateTime { date, hour, minute }` | `new` clamps the hour to 23 and the minute to 59. `shifted(part, delta)`, `typed(part, value)` |
| `DatePart` | `Day`, `Month`, `Year`, `Hour`, `Minute`; `digits()` is 4 for the year, 2 otherwise |
| `days_in_month(year, month)` | 28 to 31, with leap years |
| `typing_is_done(part, typed)` | Whether typed digits fill the part |

### Style

`DatePicker` draws the field, its parts and the calendar from a
[`DatePickerStyle`](../../crates/bk/src/styles/date_picker.rs); `.style(|s| ..)` changes
one picker. The arrows draw from the `stepper` field.

```rust
DatePicker::new("Date", moment).style(|s| DatePickerStyle { day_shape: Shape::Circle, ..s })
```

## Path control

`path_control::PathControl::new(label)` shows the parts of a path in a 22 row, separated by
chevrons. `.item(Symbol, name)` adds a part with its icon. `.on_select(|index: usize| ..)`
reports the pressed part.

### Style

`PathControl` draws from a [`PathControlStyle`](../../crates/bk/src/styles/path_control.rs):
the sizes and paints of the icons, names and chevrons.

## Image well

`image_well::ImageWell::new(label, State<Option<PathBuf>>)` is a 64 square, corner 8, that
shows the image at the path.

| Input | Effect |
|---|---|
| Image file dragged over it | A 2 accent border |
| Image file dropped | Sets the path |
| Click, Space or Enter | Opens the portal file chooser ([13](13-app.md)) |
| Delete or Backspace | Clears it |

`is_image(path)` accepts `png`, `jpg`, `jpeg`, `gif`, `webp` and `bmp`, in any case.

### Style

`ImageWell` draws from an [`ImageWellStyle`](../../crates/bk/src/styles/image_well.rs),
with the border it shows while an image is dragged over it.

## Progress

Module `progress`. They report nothing.

| Item | Build | Look |
|---|---|---|
| `ProgressView` | `ProgressView::determinate(label, value)` (0 to 1), `.tint(Tint)`, `.style(\|s\| ..)` | 8 tall bar |
| `ProgressView` | `ProgressView::indeterminate(label)` | A segment of 30 % that sweeps over 1.2 s |
| `ProgressRing` | `ProgressRing::new(label, value)` | 32 ring, 5 stroke, round ends |
| `Spinner` | `Spinner::new(label)` | 16, 12 spokes, one turn per second |

With reduced motion ([4](04-appearance-and-tokens.md)) the indeterminate bar and the spinner
stand still.

```rust
ProgressView::determinate("Copying system", 0.62)
ProgressView::indeterminate("Searching for networks")
ProgressRing::new("Downloading", 0.4)
Spinner::new("Loading")
```

### Style

`ProgressView`, `ProgressRing` and `Spinner` draw from a `ProgressViewStyle`, a
`ProgressRingStyle` and a `SpinnerStyle`, each with `.style(|s| ..)`. The
sweep of the indeterminate bar and the turn of the spinner are their
`animation`. The fields are in
[Styles and themes](../../crates/bk/src/styles/progress_view.rs).

```rust
ProgressView::determinate("Copying system", 0.62).style(|s| ProgressViewStyle { height: 4.0, ..s })
```

## Level indicators

Module `indicator`, after `NSLevelIndicator` and SwiftUI `Gauge`.

| Item | Build | Look |
|---|---|---|
| `CapacityIndicator` | `new(label, value)`, `.thresholds(warning, critical)` (default 1, 1), `.discrete(cells)` | 16 tall bar; green, then yellow at `warning`, red at `critical`. `discrete` splits it into touching cells with a darker border |
| `RelevancyIndicator` | `new(label, value)` | 12 tall track with a row of 1 px lines, as many as `value` fills |
| `Gauge` | `new(label, value, current)`, `.capacity(bool)` | 44 open ring (270°) with a marker and `current` in the center; `capacity(true)` is a closed ring that fills with the accent |
| `RatingIndicator` | `new(label, State<u32>)`, `.editable(bool)` (default `false`) | 5 filled stars 12 apart; editable only with `.editable(true)` (click or arrows) |

| Helper | Returns |
|---|---|
| `level_color(value, warning, critical)` | 0, 1 or 2: the index into `LEVEL_INDICATOR_COLORS` |
| `gauge_marker(style, value)` | The `(x, y)` center of the marker of a gauge drawn with `style` |

The gauge size, stroke and arc are estimates: Apple publishes no figures.

```rust
CapacityIndicator::new("Disk", 0.82).thresholds(0.75, 0.9)
let stars = use_state(|| 3);
RatingIndicator::new("Rating", stars).editable(true)
Gauge::new("Temperature", 0.64, "64°")
```

### Style

Each indicator has `.style(|s| ..)` and draws from its own style:
[`CapacityIndicatorStyle`](../../crates/bk/src/styles/capacity_indicator.rs), and
`RelevancyIndicatorStyle`, `RatingIndicatorStyle` and `GaugeStyle`
([fields](../../crates/bk/src/styles/relevancy_indicator.rs)).

```rust
RatingIndicator::new("Rating", stars).style(|s| RatingIndicatorStyle { star: StateValue::new(ShapeStyle::Accent), ..s })
```

## Adornments

Module `adornments`. `Badge`, `Divider` and `Avatar` are rects: they take Freya's layout and
style methods.

| Item | What it is |
|---|---|
| `Badge::new(content)` | A 20 tall capsule, destructive red, with `content` in `CAPTION` |
| `Divider::new()` | A hairline that fills the width; also in the prelude |
| `Avatar::new(initials)` | A 38 circle with the initials in `HEADLINE` |
| `Avatar::named(name)` | The same, with the first letter of each of the first two words of `name`: `initials(name)` |
| `Avatar::picture(path)` | Shows the image at `path` filling the circle, cropped in the center, instead of the initials |
| `Link::new(text)`, `.on_press(\|_\| ..)`, `.style(\|s\| ..)` | Text in the link color |

```rust
Badge::new("2")
Divider::new()
Avatar::new("JA")
Avatar::named("Jane Appleseed").picture("/var/lib/face.png")
Link::new("Learn more…")
```

### Style

`Link` draws from a [`LinkStyle`](../../crates/bk/src/styles/link.rs): its paint and
font. `Badge`, `Divider` and `Avatar` draw from a `BadgeStyle`, a
`DividerStyle` and an `AvatarStyle`, resolved when they are built; their
fields are in
[Styles and themes](../../crates/bk/src/styles/badge.rs).
To change a single one, [paint on views](../../agents/styles.md#paint-on-views) also works.

```rust
Badge::new("3").style(|s| BadgeStyle {
    background: StateValue::new(ShapeStyle::Accent),
    ..s
})
```

## Scrolling

`ScrollView` (in `controls` and the prelude) scrolls its children.

| Method | Default | Effect |
|---|---|---|
| `ScrollView::new()`, `.child(..)`, `.children(..)` | — | — |
| `.width(Size)`, `.height(Size)`, `.expanded()` | fill, fill | Size of the viewport |
| `.direction(Direction)` | `Vertical` | Scroll axis |
| `.scroll_bar(bool)` | `true` | Draws the bar |
| `.top_line(bool)` | `false` | A divider over the top edge; content slides under it |
| `.edge_effect(bool)` | `false` | Content fades under an edge over 24 (`SCROLL_BAR.edge_height`), like `scrollEdgeEffectStyle(.soft)`: the top edge while scrolled; sideways, each edge with more content past it |
| `.controller(ScrollController)` | its own | Scroll from code |
| `.refreshable(refreshing, action)` | off | A touchpad pull of the content past its start by 80 calls `action` once; while `refreshing` is true the content rests 60 down with a spinner above it, and it returns when it turns false |
| `.scroll_position(State<ScrollPosition>)` | off | Scrolls to what the app sets and writes the user's scroll back, see [Scroll position](#scroll-position) |
| `.on_scroll_geometry_change(action)` | off | Calls `action(ScrollGeometry)` when the offset or a size changes |
| `.on_scroll_phase_change(action)` | off | Calls `action((old, new))` with a `ScrollPhase` when the phase changes |

The bar is a 6 thumb that appears while scrolling and fades 800 ms later. With the
appearance setting `scroll_bars = "always"` ([4](04-appearance-and-tokens.md)) it stays,
17 wide on a faint track that takes its strip out of the content, like a legacy
scroller; `"automatic"` follows the system's scroller style ([4](04-appearance-and-tokens.md)).
The track of a `TextEditor` stays while its text fits, as an `NSTextView`'s
does; other scroll views hide it. The thumb and the track
change with Increase Contrast ([measured](../research/macos-control-metrics.md#scrollers)).
`Table`, `List` and `OutlineView` draw the same bar. A click on
the track pages toward the click, or with `scroll_bar_click = "clicked_spot"`
centers the thumb there, like the macOS Appearance setting.

A touchpad scroll (a scroll event with a gesture phase) moves the content 1:1,
stretches it past an edge by a rubber band and, on release, glides to a stop with
the distance and the path measured on a Mac Catalyst `UIScrollView`; a glide that
reaches an edge bounces. A mouse wheel steps as before, with no stretch, and with
reduced motion nothing stretches or glides. On macOS the system's momentum events
already glide, so only the stretch and the return act. The constants and their
measures are in [UIKit animations](../research/uikit-animations.md#6-scroll); `List` and `OutlineView` inherit it.
The kit owns the offset because Freya clamps it and flings linearly, while the measured stop distance is not linear in the velocity.

`use_scroll_controller()` returns a `ScrollController` (`Copy`):

| Method | Effect |
|---|---|
| `reveal_y(top, bottom)` | Scrolls the least needed to show that band |
| `scroll_to_x(x)` | Jumps to that horizontal offset |
| `scroll_by_wheel(&WheelEventData)` | Scrolls by a wheel event caught on a layer above the view |
| `scroll_to(ScrollPosition, Animation)` | Goes to a position; `Animation::None` jumps |
| `content_offset()`, `geometry()`, `phase()` | The offset (positive when scrolled), the `ScrollGeometry` and the `ScrollPhase` now |

```rust
ScrollView::new().height(Size::px(120.0)).child(list)
```

### Scroll position

`ScrollPosition` is SwiftUI's: `at(x, y)`, `edge(Edge)` and `id(id, anchor)` say where; `point()` and
`is_positioned_by_user()` read it. `.id("row-3")` (from `IdentityModifiers`, in the prelude) names a
view inside a `ScrollView`; the id is a `String`. The anchor is an `Alignment`: `Start`, `Center` or
`End` put that point of the view on the same point of the viewport; `None` scrolls the least that shows
the view. An id the view does not have does nothing.

Pass a `State<ScrollPosition>` to `.scroll_position(..)` and the view scrolls when the app sets it. Set
it inside `with_animation` to animate the scroll. When the user scrolls, the view writes
`ScrollPosition::at(..)` back, with `is_positioned_by_user()` true. `scroll_to(position, animation)` of a
`ScrollController` is the same without the state.

`ScrollPhase` is `Idle`, `Tracking`, `Interacting`, `Decelerating` or `Animating`. A touchpad pull or a
finger gives `Interacting` and then `Decelerating`; `scroll_to` with an animation gives `Animating`.
`Tracking` is set when a gesture begins and is replaced in the same event, so a callback sees
`Interacting`. A mouse wheel gives no phase, and on macOS the system's momentum counts as `Interacting`.

The program is [`recipes/scroll_position.rs`](../../agents/recipes/scroll_position.rs).
Not built: `scrollTargetBehavior`, `defaultScrollAnchor` and a position set before the first layout.

### Style

`ScrollView` draws its bar, edge fade and top line from a `ScrollViewStyle`:
`.style(|s| ..)` for one view, `.control_style(|s: ScrollViewStyle| ..)` for
the views inside another. The fields are in
[Styles and themes](../../crates/bk/src/styles/scroll_view.rs).

## Drag source

Module `drag`. It makes an element the source of a drag to another app. Receiving drops and
the clipboard are in [13](13-app.md).

```rust
let drag = use_drag_source();
drag.attach(rect(), || DragPayload::Files(paths.clone()))
```

| Item | What it is |
|---|---|
| `use_drag_source()` | Returns a `DragSource` (`Copy`) |
| `DragSource::attach(element, payload)` | Starts a drag when the pointer moves more than `DRAG.threshold` (8) with the primary button down; `payload` runs then |
| `DragPayload::Files(Vec<PathBuf>)` | Offered as `URI_LIST` and as plain text |
| `DragPayload::Text(String)` | Offered as plain text |
| `DragPayload::Html { text, html }` | Offered as plain text and as `text/html` (`public.html` on macOS) |
| `DragPayload::Image(Vec<u8>)` | PNG bytes; offered as `image/png` (PNG and TIFF on macOS) |
| `URI_LIST` | `"text/uri-list"` |

A type of the app joins in through `Transferable` ([13](13-app.md#clipboard)): `drag.attach(rect(), move || note.payload())`.
On macOS the drag is an `NSDraggingSession` started from the content view with the mouse event the pointer is making,
and it shows the image, the file icon or a document symbol under the pointer. Dropping typed data onto the app is not
built: only files arrive ([13](13-app.md#drag-and-drop)).

## Canvas

`canvas::Canvas::new(revision, painter)` fills its parent and hands the area to `painter`, which
draws with Skia in logical pixels. It repaints only when `revision` (`u64`) changes. Use it for
what is not rectangles or text, such as a terminal grid.

`CanvasContext` fields: `canvas` (the Skia canvas, clipped and scaled), `width`, `height`,
`scale` and `fonts` (the font collection).

```rust
Canvas::new(revision, move |ctx: &mut CanvasContext| { ctx.canvas.draw_rect(..); })
```

A `Canvas` has no style: the painter draws everything.

## Shapes

`Rectangle`, `RoundedRectangle::new(corner_radius)`, `Capsule`, `Circle`, `Ellipse` and `Path` (in
`shapes` and the prelude) fill their frame; give them a size with `.width(..)`, `.height(..)` or a
`Frame`. They draw with a [Canvas](#canvas): Freya has no path element, and its `rect` cannot draw an ellipse.

| Method | Effect |
|---|---|
| `.fill(ShapeStyle)` | Fills the shape with a color, a role, the accent or a gradient |
| `.stroke_border(ShapeStyle, line_width)` | The outline inside the frame, as `strokeBorder` (every shape but `Path`) |
| `Path::new().move_to(p)`, `.add_line(p)`, `.add_curve(to, c1, c2)`, `.close_subpath()` | A path in the points of the frame; `.stroke(ShapeStyle, line_width)` draws it centered |

A `RoundedRectangle` has the continuous corner of the system
(`CORNER_SMOOTHING`, [Corners](04-appearance-and-tokens.md#corners)). A `Circle` is centered in a frame that is not
square. A `ShapeStyle::Material` fills with its tint and does not blur.

`.clip_shape(Shape)` (`ShapeModifiers`, in the prelude) clips a view and its children to a `Shape` of
`styles`: `Rectangle`, `RoundedRectangle { .. }`, `UnevenRoundedRectangle { .. }`, `Capsule` or `Circle`.
Freya clips with a rounded rectangle, so an ellipse or a `Path` cannot clip, and a circle clip of a view
that is not square is a capsule.

The program is [`recipes/shapes.rs`](../../agents/recipes/shapes.rs).

## Accessibility modifiers

SwiftUI: `.accessibilityLabel`, `.accessibilityHint`, `.accessibilityValue`,
`.accessibilityHidden`, `.accessibilityIdentifier`, `.accessibilityElement(children:)`
and `.accessibilityAddTraits`. The program is
[`recipes/accessibility.rs`](../../agents/recipes/accessibility.rs).

`AccessibilityModifiers` is in the prelude and works on every view and control.

| Modifier | Sets | macOS AX attribute |
|---|---|---|
| `accessibility_label(text)` | The name; wins over the text Freya takes from the children | `AXTitle` |
| `accessibility_hint(text)` | What the control does | `AXHelp` |
| `accessibility_value(text)` | The current value | `AXValue` |
| `accessibility_hidden(true)` | Removes the view and its children from the tree | none |
| `accessibility_identifier(id)` | A stable id for tests and scripts | `AXIdentifier` |
| `accessibility_element(AccessibilityChildBehavior::Combine)` | One element that speaks its children's labels, `Ignore` hides the children, `Contain` makes a group | role and children |
| `accessibility_add_traits(AccessibilityTraits::IS_HEADER \| ..)` | The role: `IS_BUTTON`, `IS_HEADER`, `IS_LINK`, `IS_IMAGE`, `IS_STATIC_TEXT`; `IS_SELECTED` selects | `AXRole` |

1. Call the modifier on the view or control: `Button::new("+").accessibility_label("Add a task")`.
2. A `TextField`, `SearchField` and `TextEditor` expose their text as the value and their placeholder as the name when you give no label.
3. On macOS each window publishes its tree to VoiceOver and to the AX API ([18](18-platform.md#accessibility)). To check it, read the title, help and identifier of each element in Xcode's Accessibility Inspector.
4. The tree is built only when a client asks, so a window with no screen reader costs nothing.

The kit applies the modifiers to the finished tree because Freya replaces the label of a node that has text.

On Linux AT-SPI reads the same resolved tree, `accessibility_element` included.

## Building a new control

1. Check that no control does it already (rule 1 of `AGENTS.md`). A new control composes kit
   primitives; it does not redraw menus, the sidebar or traffic lights (ADR 0047).
2. Call `use_interaction()` at the top of `render`.
3. `interaction.attach(rect(), enabled)` gives the element an accessibility id, focus,
   hover and press tracking. Set its `a11y_role` and `a11y_alt`.
4. Read `hovered()`, `pressed()` (held and still under the pointer), `held()`, `focused()` and
   `keyboard_focused()` to choose colors.
5. Wrap the result in `interaction.focus_ring(element, corner_radius)`: an outer border of
   `FOCUS_RING.width` that follows the element's corners. Use
   `interaction.inner_focus_ring(element)` instead when the element fills its parent and an
   outer ring would be clipped, like the rows of `Form` and the cells of `Table`.
   `element.focus_ring(radius, color)` ([`ViewModifiers`](07-menus-and-popovers.md#focus_ring))
   draws the same ring in another color, as a selection mark.
6. Paint only with `Palette::current()` and `ctdesign` tokens ([4](04-appearance-and-tokens.md)).
   New metrics go to `ctdesign`.
7. If the control edits a value, take its `State<T>` in `new` and write it
   ([the controlled pattern](#the-controlled-pattern)).
8. Inside `bk`, take the [modifiers](#modifiers-on-controls): hold
   `overrides: Overrides` (start it with `Overrides::default()`), add
   `forward_overrides!(Pill);` and end `render` with `self.overrides.apply(root)`, where `root` is
   the outermost node. A control that renders another control ends with
   `self.overrides.delegate(inner)` instead.

```rust
#[derive(PartialEq)]
struct Pill {
    label: &'static str,
}

impl Component for Pill {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let background = if interaction.hovered() {
            palette.control_hover
        } else {
            palette.control_background
        };
        let pill = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .height(Size::px(BUTTON.height))
            .corner_radius(BUTTON.height / 2.0)
            .background(color(background))
            .child(text(self.label, BODY, palette.label));
        interaction.focus_ring(pill, BUTTON.height / 2.0)
    }
}
```

`Interaction::request_focus()` moves the focus to the control from code.

Next: [6. Text and editing](06-text-and-editing.md).
