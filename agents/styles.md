# Styles and themes: prompt for AI agents

**Goal:** let an AI agent change how a control looks (color, translucency,
blur, shape, corners, size, shadow, layout, direction, animation, color
algorithm) for one control, a container or the whole app, or build a theme
that does not look like Tengen.
**Scope:** the styling system of [ADR 0064](../docs/decisions/0064-styles-and-themes.md).
Writing an app in general is in [`prompt.md`](prompt.md); colors and tokens
are in [4. Appearance and tokens](../docs/barrierkit/04-appearance-and-tokens.md).
**Reader:** an AI agent. Read it whole before styling anything.
**Input:** a description of the look. **Output:** style code at the right
level, checked against [Rules](#rules).

---

You are styling a BarrierKit app. Every control draws itself from one style
value. You never draw a control again by hand to change its look: you change
its style.

## Steps

1. Decide the level from the request (see [Levels](#levels)):
   - "this slider" → instance, `.style(..)`;
   - "the sliders in this panel" → container, `.control_style(..)`;
   - "every slider in the app", "a new theme" → theme, `Theme`;
   - the color, background or shadow of one `Text`, a stack, `rect()`,
     `Badge` or `Divider` → [Paint on views](#paint-on-views); the
     defaults of every `Text`, `Badge`, `Divider`, `Avatar` or `Image` →
     their view styles (`TextStyle` and the rest, in
     [`crates/bk/src/styles/`](../crates/bk/src/styles/)).
2. Check in [Which controls take a style](#which-controls-take-a-style) that
   the control supports it. If not, say so; do not imitate it with `rect()`.
3. Read the fields and Tengen values of the style in the rustdoc of its
   struct ([The fields of a style](#the-fields-of-a-style)).
4. Write the override as a function of the inherited style that names only
   what changes and ends with `..s`.
5. Pick values with [Value types](#value-types): a `Role` or `Accent` when the
   color must follow the system, `Color` only when it must not.
6. Leave accessibility to the kit ([Accessibility](#accessibility)).
7. Build with `cargo clippy -- -D warnings` inside `nix develop`.

## Levels

Each level receives the result of the one before and returns a new style.
Later levels win, field by field.

| Order | Level | Code |
|---|---|---|
| 1 | Tengen (default) | `SliderStyle::tengen()` |
| 2 | Theme | `Application::new("app").theme(Theme::tengen().style(\|s: SliderStyle\| …))`, or `WindowOptions::theme` for `run_window` |
| 3 | Container | `VStack::new().child(..).control_style(\|s: SliderStyle\| …)` (outermost first) |
| 4 | Instance | `Slider::stepped("Volume", 10).style(\|s\| …)` |
| 5 | Accessibility | applied by the kit, always |

`Theme::style` and `.control_style` are generic: the type of the closure's
parameter (`|s: SliderStyle|`) picks the control they change. Write the type;
without it the code does not compile. A `Theme` holds overrides of any number
of controls; chain one `.style(..)` per control.

`.control_style` wraps the view in a `StyleScope`, so it comes last, after
the children. It reaches every control of that style rendered inside, also
inside nested components.

A popup is a window of its own: a menu, a submenu, a popover, a tooltip or the
calendar of a `DatePicker`. When a view opens one, the kit hands it that
view's theme and container styles, so the popup's content resolves its styles
like the view that opened it. `Slider::tint(Tint)` is an instance override of
`fill`.

✗ Copy Tengen's numbers into a style:

```rust
Slider::stepped("Volume", 10).style(|_| SliderStyle { track_height: 6.0, width: 220.0, /* … every field … */ })
```

✓ Change only what you mean and keep the rest:

```rust
Slider::stepped("Volume", 10).style(|s| SliderStyle { track_height: 10.0, ..s })
```

## Value types

All of them are in `bk::styles` and in the prelude.

| Type | Use it for | Values |
|---|---|---|
| `ShapeStyle` | any paint | `Role(Role)` follows light/dark/contrast · `Accent` follows the accent (system, app or `Backdrop`; inactive window) · `Color(Rgba)` fixed · `Gradient(Gradient)` · `Backdrop` the gradient of the surrounding `Backdrop`, the accent without one · `Material(Thickness)` translucency with blur · `Clear` nothing |
| `Role` | a semantic color | every color of `Palette` but `accent` (that is `ShapeStyle::Accent`): `Label`, `SecondaryLabel`, `TertiaryLabel`, `Separator`, `ControlBorder`, `WindowBackground`, `ContentBackground`, `ControlBackground`, `ControlHover`, `AccentInactive`, `OnAccent`, `Destructive`, `RaisedBackground`, `SunkenBackground`, `Scrim`, `SelectedControl`, `CardBorder` |
| `Gradient` | a gradient paint | `Gradient::new(angle, stops)`; `angle` in degrees, `stops: Vec<(ShapeStyle, f32)>` with positions 0–1 |
| `Thickness` | how translucent a material is | `UltraThin`, `Thin`, `Regular`, `Thick`, `UltraThick`; opacity and blur from `MATERIAL` ([4](../docs/barrierkit/04-appearance-and-tokens.md#materials)) |
| `StateValue<T>` | a value that changes with the state | `StateValue::new(normal)` and `.hovered(..)`, `.pressed(..)`, `.focused(..)`, `.disabled(..)`; the first set one wins in the order disabled, pressed, hovered, focused, then `normal` |
| `ControlState` | the state of a control | `hovered`, `pressed`, `focused`, `disabled` (`bool`) |
| `Shape` | the outline of a part | `Rectangle`, `RoundedRectangle { radius, style }`, `UnevenRoundedRectangle { top_leading, bottom_leading, bottom_trailing, top_trailing, style }`, `Capsule`, `Circle` (a capsule on a part that is not square) |
| `CornerStyle` | the corner curve | `Continuous` (Apple's smooth corner), `Circular` |
| `ShadowStyle` | one shadow | `color: Rgba`, `x`, `y`, `blur`, `spread`, `inner`; a part takes a `Vec` |
| `Animation` | how a value change moves | `None`, `Ease { duration_ms }` (cubic ease-out), `Spring { duration_ms, bounce }`, `Custom { duration_ms, curve }` with `curve: fn(f32) -> f32` from 0–1 to 0–1 |
| `ColorEffect` | your own color algorithm | `fn(Rgba, ControlState) -> Rgba`, applied to every paint of the control; Tengen's `system_effect` returns the color unchanged |
| `Axis` | orientation | `Horizontal`, `Vertical` |
| `Edge` | where labels go | `Top`, `Bottom`, `Leading`, `Trailing` |
| `LayoutDirection` | reading direction | `LeftToRight`, `RightToLeft` |

## The fields of a style

Every control style has the same groups. The names of the parts change per
control; the groups do not. A control leaves out a group it does not draw.

| Group | Fields (slider) |
|---|---|
| Size | `width`, `track_height`, `knob_size`, `tick_size`, `tick_gap`, `label_gap` |
| Shape | `track_shape`, `knob_shape`, `tick_shape` |
| Paint | `track`, `fill`, `knob`, `tick`, `label` |
| Border and shadow | `knob_border`, `knob_shadows` |
| Effects | `opacity`, `disabled_opacity`, `effect` |
| Layout | `axis`, `labels`, `layout_direction` |
| Typography | `label_font` |
| Motion | `animation` |

The fields of each style, their Tengen values, its presets and the states its
control enters are in the rustdoc of its struct, in
[`crates/bk/src/styles/`](../crates/bk/src/styles/): read the file, or run
`cargo doc -p bk --open` inside `nix develop`. The
[table below](#which-controls-take-a-style) gives the style of each control.

A `Capsule` or `Circle` needs the size of the part. Where the width is only
known after layout (a button, a text field, a segmented control or a tool
group), it rounds by half the height.

A `StateValue` value for a state the control never enters never applies.

## MenuStyle

`MenuStyle` is `ctdesign::MenuPalette`: every color and metric of a menu, a
pop-up menu, the popover background and the tooltip. Its fields and their
groups are in [4. Appearance and tokens](../docs/barrierkit/04-appearance-and-tokens.md#token-types).

| Tengen | Comes from |
|---|---|
| A menu (`MenuView`, `.context_menu`, submenus) | `menu_palette(resolved, surface)` |
| A pop-up menu (`PopUpButton`, `ComboBox`, `MenuButton`), a `Popover`, `.help` and the `DatePicker` calendar | `popover_palette(resolved, Toplevel)`: the `MENU_POPOVER` metrics |

The colors are `Rgba`, not `ShapeStyle`, because a menu is drawn on its own
surface. The override runs on every render, after the palette of the current
appearance is built, so `..s` keeps following light, dark and contrast, and a
value read from `Palette::current()` inside the closure follows it too.

```rust
Theme::tengen().style(|s: MenuStyle| MenuStyle {
    item_radius: 0.0,
    hover_bg: Palette::current().destructive,
    ..s
})
```

`MenuView::style(|s| …)` changes one menu and its submenus.

## Examples

This slider red, taller, flat and with almost square corners; the others stay
normal:

```rust
Slider::stepped("Volume", 10).style(|s| SliderStyle {
    fill: StateValue::new(ShapeStyle::Role(Role::Destructive)),
    track_height: 10.0,
    knob_size: (32.0, 14.0),
    knob_shape: Shape::RoundedRectangle { radius: 3.0, style: CornerStyle::Continuous },
    track_shape: Shape::RoundedRectangle { radius: 2.0, style: CornerStyle::Continuous },
    ..s
})
```

Every slider in a panel vertical, with a translucent track:

```rust
HStack::new()
    .child(Slider::stepped("Bass", 10))
    .child(Slider::stepped("Treble", 10))
    .control_style(|s: SliderStyle| SliderStyle {
        axis: Axis::Vertical,
        track: StateValue::new(ShapeStyle::Material(Thickness::Thin)),
        ..s
    })
```

A theme for the whole app with its own color algorithm:

```rust
use bk::prelude::*;

fn darker_when_pressed(color: Rgba, state: ControlState) -> Rgba {
    if state.pressed { color.darken(0.2) } else { color }
}

fn main() -> Result<(), RunError> {
    Application::new("mixer")
        .theme(Theme::tengen().style(|s: SliderStyle| SliderStyle { effect: darker_when_pressed, ..s }))
        .window("Mixer", || Slider::stepped("Volume", 10).value(4))
        .run()
}
```

### By style

ButtonStyle:

```rust
Button::new("Save").style(ButtonStyle::bordered_prominent)
Button::new("Delete").style(|s| ButtonStyle {
    background: StateValue::new(ShapeStyle::Role(Role::Destructive)),
    ..s.bordered_prominent()
})
```

CheckboxStyle:

```rust
Checkbox::new("Remember me").style(|s| CheckboxStyle { label_position: Some(Edge::Leading), ..s })
Switch::new("Wi-Fi").style(|s| SwitchStyle { label_position: Some(Edge::Leading), ..s })
```

LevelSliderStyle:

```rust
LevelSlider::new("Volume").style(|s| s.fat(Symbol::SpeakerWave))
```

RadioGroupStyle:

```rust
RadioGroup::new("Size").option("Small").option("Large").style(|s| RadioGroupStyle {
    axis: Axis::Horizontal,
    label_position: Some(Edge::Leading),
    ..s
})
```

ToolbarSearchStyle:

```rust
ToolbarSearch::new(query, "Search")
    .control_style(|s: SearchFieldStyle| SearchFieldStyle { shape: Shape::Rectangle, ..s })
```

ComboBoxStyle:

```rust
ComboBox::new(shell, "Shell")
    .style(|s| ComboBoxStyle { button: StateValue::new(ShapeStyle::Clear), ..s })
    .control_style(|s: TextFieldStyle| TextFieldStyle { shape: Shape::Capsule, ..s })
```

LinkStyle:

```rust
Link::new("Learn more…").style(|s| LinkStyle {
    label: StateValue::new(ShapeStyle::Accent).hovered(ShapeStyle::Role(Role::Label)),
    ..s
})
```

TextStyle, BadgeStyle, DividerStyle, AvatarStyle and ImageStyle:

```rust
Theme::tengen()
    .style(|s: TextStyle| TextStyle { font: CALLOUT, ..s })
    .style(|s: BadgeStyle| BadgeStyle {
        background: StateValue::new(ShapeStyle::Accent),
        ..s
    })
```

ChartStyle:

```rust
Theme::tengen().style(|s: ChartStyle| ChartStyle {
    series: vec![ShapeStyle::Accent, ShapeStyle::Role(Role::SecondaryLabel)],
    bar_radius: 4.0,
    ..s
})
```

TitlebarStyle and FullscreenBarStyle:

```rust
Titlebar::new().style(|s| TitlebarStyle {
    title_alignment: Alignment::Start,
    hairline: None,
    ..s
})
```

WindowChromeStyle:

```rust
WindowChrome::new(content).style(|s| WindowChromeStyle {
    layout: TitlebarLayout::Rail,
    reveal: Reveal::Drawer,
    resting_edge: RestingEdge::Accent,
    ..s
})
```

SidebarStyle, SidebarItemStyle, SidebarSectionStyle, SidebarAccountStyle and SymbolTileStyle:

```rust
Theme::tengen().style(|s: SidebarItemStyle| SidebarItemStyle {
    selected_background: StateValue::new(ShapeStyle::Role(Role::SelectedControl)),
    selected_label: StateValue::new(ShapeStyle::Role(Role::Label)),
    shape: Shape::Capsule,
    ..s
})
```

FormStyle, GroupBoxStyle and DisclosureGroupStyle:

```rust
Form::new()
    .section(Section::titled("Network").row(Row::value("Status", "Connected")))
    .style(|s| FormStyle {
        shape: Shape::RoundedRectangle { radius: 16.0, style: CornerStyle::Continuous },
        background: StateValue::new(ShapeStyle::Material(Thickness::Thin)),
        ..s
    })
```

TableStyle, CollectionViewStyle and OutlineViewStyle:

```rust
Table::new("Files", vec!["Name", "Size"])
    .rows(rows)
    .style(|s| TableStyle {
        alternate_row_background: StateValue::new(ShapeStyle::Clear),
        selected_row_background: StateValue::new(ShapeStyle::Role(Role::SelectedControl)),
        selected_label: StateValue::new(ShapeStyle::Role(Role::Label)),
        ..s
    })
```

SheetStyle, AlertStyle and ContentUnavailableViewStyle:

```rust
Sheet::new(content).style(|s| SheetStyle {
    background: StateValue::new(ShapeStyle::Material(Thickness::Regular)),
    border: None,
    ..s
})
```

## Paint on views

Views that are not controls take the same `ShapeStyle` through
`ViewModifiers`, in the prelude:

| Modifier | Views | Effect |
|---|---|---|
| `.foreground_style(ShapeStyle)` | `Text`, `label()`, `paragraph()` | The color of the text; a `Gradient` or `Backdrop` paints the glyphs; a `Material` is its tint, without blur |
| `.background_style(ShapeStyle)` | `VStack`, `HStack`, `rect()`, `Badge`, `Divider` | The background; a `Material` adds its blur of what is behind |
| `.shadows(Vec<ShadowStyle>)` | the same | Adds the shadows, in order |

The paint resolves when the view is built, against the palette, accent,
`Backdrop` and appearance of the component that builds it, with the same
resolver as the controls: a `Material` is solid when the user reduces
transparency.

These modifiers change one view. The defaults of every `Text`, `Badge`,
`Divider`, `Avatar` and `Image` are their
view styles (`TextStyle`, `BadgeStyle`, `DividerStyle`, `AvatarStyle`, `ImageStyle`).

✗ Color a stack to change its texts: a `Text` keeps its own color.

```rust
VStack::new().child(Text::new("Saved")).foreground_style(ShapeStyle::Accent)
```

✓ Color each `Text`:

```rust
VStack::new()
    .child(Text::new("Saved").foreground_style(ShapeStyle::Accent))
    .background_style(ShapeStyle::Material(Thickness::Thin))
    .shadows(vec![ShadowStyle {
        color: Rgba { a: 0.2, ..Rgba::rgb8(0, 0, 0) },
        x: 0.0,
        y: 2.0,
        blur: 8.0,
        spread: 0.0,
        inner: false,
    }])
```

Corner radii stay with `.corner_radius(..)`: a `Shape` is not a view
modifier, because `Capsule` and `Circle` need the size of the view, which is
only known after layout.

## Accessibility

The kit applies these after every level; you cannot turn them off:

1. Reduce motion: every `Animation` becomes `None`.
2. Reduce transparency: every `Material` becomes its solid color, without
   blur. So do the kit's own glass surfaces: the glass full-screen bars and
   `TopGlass` become the solid bar, and the media `Material` and
   `GlassContainer` become the opaque `MEDIA.material_tint`.
3. Keyboard focus: the focus ring is always drawn; no field turns it off.

Contrast is your job when you use `ShapeStyle::Color`: text on a fixed color
needs 4.5:1 and control parts 3:1 (WCAG 2.2, 1.4.3 and 1.4.11). Roles and
`Accent` already meet it.

## Which controls take a style

| Control | Style | Since |
|---|---|---|
| `Slider` | `SliderStyle` | ADR 0064 pilot |
| `Button` | `ButtonStyle` | 2026-11-07 |
| `Switch` | `SwitchStyle` | 2026-11-07 |
| `Checkbox` | `CheckboxStyle` | 2026-11-07 |
| `SegmentedControl` | `SegmentedControlStyle` | 2026-11-07 |
| `TextField` | `TextFieldStyle` | 2026-11-07 |
| `SearchField` | `SearchFieldStyle` | 2026-11-07 |
| `SecureField` | `SecureFieldStyle` | 2026-11-07 |
| `ToolGroup` | `ToolGroupStyle` | 2026-11-07 |
| `LevelSlider` | `LevelSliderStyle` | 2026-11-07 |
| `RadioGroup` | `RadioGroupStyle` | 2026-11-07 |
| `ColorWell` | `ColorWellStyle` | 2026-11-07 |
| `ChoiceCards` | `ChoiceCardsStyle` | 2026-11-07 |
| `IconButton`, `ToolbarOverflow` | `IconButtonStyle` | 2026-11-07 |
| `Stepper`, the arrows of `DatePicker` | `StepperStyle` | 2026-11-07 |
| `ToggleGroup` | `ToggleGroupStyle` | 2026-11-07 |
| `ToolbarSearch` | `ToolbarSearchStyle`; its field and button take `SearchFieldStyle` and `IconButtonStyle` | 2026-11-07 |
| `MenuButton`, `ShareLink` | `MenuButtonStyle` | 2026-11-07 |
| `ComboButton` | `ComboButtonStyle` | 2026-11-07 |
| `HelpLink` | `HelpLinkStyle` | 2026-11-07 |
| `ControlGroup` | `ControlGroupStyle` | 2026-11-07 |
| `PopUpButton` | `PopUpButtonStyle` | 2026-11-07 |
| `ComboBox` | `ComboBoxStyle`; its field takes `TextFieldStyle` | 2026-11-07 |
| `TokenField`, the tokens of `SearchField` | `TokenFieldStyle` | 2026-11-07 |
| `DatePicker` | `DatePickerStyle`; its arrows take `StepperStyle` | 2026-11-07 |
| `PathControl` | `PathControlStyle` | 2026-11-07 |
| `ImageWell` | `ImageWellStyle` | 2026-11-07 |
| `ColorPicker` | `ColorPickerStyle` | 2026-11-07 |
| `FontPicker` | `ButtonStyle`; the rows of its panel take `OutlineViewStyle` | 2026-11-07 |
| `ProgressView` | `ProgressViewStyle` | 2026-11-07 |
| `ProgressRing` | `ProgressRingStyle` | 2026-11-07 |
| `Spinner` | `SpinnerStyle` | 2026-11-07 |
| `CapacityIndicator` | `CapacityIndicatorStyle` | 2026-11-07 |
| `RelevancyIndicator` | `RelevancyIndicatorStyle` | 2026-11-07 |
| `RatingIndicator` | `RatingIndicatorStyle` | 2026-11-07 |
| `Gauge` | `GaugeStyle` | 2026-11-07 |
| `MediaButton` | `MediaButtonStyle` | 2026-11-07 |
| `RoundButton` | `RoundButtonStyle` | 2026-11-07 |
| `Link` | `LinkStyle` | 2026-11-07 |
| `Text` | `TextStyle` | 2026-11-07 |
| `Badge` | `BadgeStyle` | 2026-11-07 |
| `Divider` | `DividerStyle` | 2026-11-07 |
| `Avatar` | `AvatarStyle` | 2026-11-07 |
| `Image` | `ImageStyle` | 2026-11-07 |
| `Material` | `MaterialStyle` | 2026-11-07 |
| `TopGlass` | `TopGlassStyle` | 2026-11-07 |
| `GlassContainer` | `GlassContainerStyle` | 2026-11-07 |
| `Backdrop` | `BackdropStyle` | 2026-11-07 |
| `ScrollView` | `ScrollViewStyle` | 2026-11-07 |
| `TextEditor` | `TextEditorStyle`; its find bar takes `SearchFieldStyle`, `IconButtonStyle` and `ButtonStyle` | 2026-11-07 |
| `RuleEditor` | `RuleEditorStyle`; its rows take `PopUpButtonStyle`, `TextFieldStyle` and `IconButtonStyle` | 2026-11-07 |
| `PrintPreview` | `PrintPreviewStyle` | 2026-11-07 |
| `Chart`, `SectorChart`, `HeatMap`, `StreamChart`, `Chart3D` | `ChartStyle` | 2026-11-07 |
| `Titlebar` | `TitlebarStyle`; its traffic lights take `TrafficLightsStyle` | 2026-11-07 |
| The full-screen bars (`FullscreenBar`) | `FullscreenBarStyle`, theme and container only | 2026-11-07 |
| `TrafficLights` | `TrafficLightsStyle` | 2026-11-07 |
| `WindowChrome` | `WindowChromeStyle`, with its layout and reveal | 2026-11-07 |
| `Sidebar` | `SidebarStyle` | 2026-11-07 |
| `SidebarItem` | `SidebarItemStyle`; its tile takes `SymbolTileStyle` | 2026-11-07 |
| `SidebarSection` | `SidebarSectionStyle` | 2026-11-07 |
| `SidebarAccount` | `SidebarAccountStyle`; its avatar takes `AvatarStyle` | 2026-11-07 |
| `SymbolTile` | `SymbolTileStyle` | 2026-11-07 |
| `SplitView` | `SplitViewStyle` | 2026-11-07 |
| `SidebarToggle` | `IconButtonStyle` | 2026-11-07 |
| `SplitPanes` | `SplitPanesStyle` | 2026-11-07 |
| `FloatingPane` | `FloatingPaneStyle` | 2026-11-07 |
| `Form` | `FormStyle`, for its sections and rows; the lines between rows take `DividerStyle` | 2026-11-07 |
| `GroupBox` | `GroupBoxStyle` | 2026-11-07 |
| `DisclosureGroup` | `DisclosureGroupStyle` | 2026-11-07 |
| `Table` | `TableStyle`, for its header cells and rows | 2026-11-07 |
| `CollectionView` | `CollectionViewStyle`, for its tiles | 2026-11-07 |
| `OutlineView`, `Browser` | `OutlineViewStyle` | 2026-11-07 |
| `TabView` | `TabViewStyle`, for its tabs | 2026-11-07 |
| `WindowTabs` | `WindowTabsStyle`, for its tabs | 2026-11-07 |
| `TextView` | `TextViewStyle` | 2026-11-07 |
| `Sheet` | `SheetStyle` | 2026-11-07 |
| `Alert` | `AlertStyle`; its sheet takes `SheetStyle`, its buttons `ButtonStyle` and its checkbox `CheckboxStyle` | 2026-11-07 |
| `ContentUnavailableView` | `ContentUnavailableViewStyle` | 2026-11-07 |
| `NavigationButtons` | `ToolGroupStyle` and `IconButtonStyle` | 2026-11-07 |
| `SettingsWindow` | `SettingsWindowStyle`, for its gaps and search popover; its parts take their own styles | 2026-11-07 |
| `AboutView` | `AboutViewStyle` | 2026-11-07 |
| `LanguagePicker` | `PopUpButtonStyle` | 2026-11-07 |
| `VideoView` | `VideoViewStyle` (feature `video`) | 2026-11-07 |
| `MenuView` and its submenus, pop-up menus, `Popover`, `.help` | `MenuStyle` | 2026-11-07 |

`VStack`, `HStack`, `Scrim` and `Canvas` take no style: they are styled with
the [paint on views](#paint-on-views) and Freya's methods.

Every style implements `ControlStyle`: `tengen()` and, when its control
animates or must change under an accessibility setting, `accessible`. A style
with `animation` turns it off under *Reduce motion* through `reduce_motion`. The rollout order is in
[ADR 0064](../docs/decisions/0064-styles-and-themes.md#5-rollout).

## Rules

1. Override with `|s| Style { field: value, ..s }`; never rebuild a whole style.
   In `Theme::style` and `.control_style`, write the parameter's type:
   `|s: SliderStyle|`.
2. Follow the system with `Role`, `Accent` or `Backdrop`; use `Color` only for
   a color that must stay fixed.
3. New default values go to `ctdesign`, never into `bk`.
4. Do not imitate a control with `rect()` to change its look.
5. Put a style on the smallest level that covers the request.
6. Every name here is in the [glossary](../docs/barrierkit/20-glossary.md).
