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
     their [view styles](#textstyle-badgestyle-dividerstyle-avatarstyle-and-imagestyle).
2. Check in [Which controls take a style](#which-controls-take-a-style) that
   the control supports it. If not, say so; do not imitate it with `rect()`.
3. Write the override as a function of the inherited style that names only
   what changes and ends with `..s`.
4. Pick values with [Value types](#value-types): a `Role` or `Accent` when the
   color must follow the system, `Color` only when it must not.
5. Leave accessibility to the kit ([Accessibility](#accessibility)).
6. Build with `cargo clippy -- -D warnings` inside `nix develop`.

## Levels

Each level receives the result of the one before and returns a new style.
Later levels win, field by field.

| Order | Level | Code |
|---|---|---|
| 1 | Tengen (default) | `SliderStyle::tengen()` |
| 2 | Theme | `Application::new("app").theme(Theme::tengen().style(\|s: SliderStyle\| …))` |
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
control; the groups do not. A control leaves out a group it does not draw:
only `BackdropStyle`, `ButtonStyle`, `ChartStyle`, `GlassContainerStyle`,
`HelpLinkStyle`, `IconButtonStyle`, `LevelSliderStyle`, `MediaButtonStyle`,
`ProgressViewStyle`, `RoundButtonStyle`, `ScrollViewStyle`,
`SidebarSectionStyle`, `SidebarStyle`, `SliderStyle`, `SpinnerStyle`,
`SplitViewStyle`, `SwitchStyle` and `WindowChromeStyle` have `animation`.

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

`SliderStyle::tengen()` builds every field from the tokens:

| Field | Type | Tengen |
|---|---|---|
| `width` | `f32` | `SETTINGS.strength.width`, the length along the axis |
| `track_height` | `f32` | `SETTINGS.strength.track` |
| `knob_size` | `(f32, f32)` | `(SLIDER.knob_width, SLIDER.knob_height)`: along the axis, across it |
| `tick_size`, `tick_gap`, `label_gap` | `f32` | `SETTINGS.strength.{tick, tick_gap, label_gap}` |
| `track_shape` | `Shape` | `Capsule`; the fill takes the same shape |
| `knob_shape`, `tick_shape` | `Shape` | `Capsule`, `Circle` |
| `track`, `fill`, `knob`, `tick`, `label` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, `Accent`, `Color(SWITCH.knob_color)`, `Role(ControlBorder)`, `Role(Label)` |
| `knob_border` | `Option<(ShapeStyle, f32)>` | `None`; paint and width, drawn inside the knob in one color |
| `knob_shadows` | `Vec<ShadowStyle>` | `[KNOB_SHADOW]` |
| `opacity` | `f32` | `1.0`, for every part |
| `disabled_opacity` | `f32` | `SWITCH.disabled_opacity`, for the track, ticks and labels |
| `effect` | `ColorEffect` | `system_effect` |
| `axis` | `Axis` | `Horizontal`; `Vertical` grows upwards |
| `labels` | `Option<Edge>` | `Some(Edge::Bottom)`; the ticks sit on the same side, below when `None` hides the labels and their space |
| `layout_direction` | `LayoutDirection` | `LeftToRight`; `RightToLeft` grows to the left and swaps ← → |
| `label_font` | `Font` | `CAPTION` |
| `animation` | `Animation` | `None`: the knob jumps to the new value, as in Tengen; set `Ease { duration_ms: DURATION_FAST_MS }` or a `Spring` to animate the knob and the fill |

On a horizontal slider `Leading` is `Top` and `Trailing` is `Bottom`; on a
vertical one `Top` is `Leading` and `Bottom` is `Trailing`. The arrow keys
follow the axis: ↑ always adds one step and ↓ removes one.

A `Capsule` or `Circle` needs the size of the part. Where the width is only
known after layout (a button, a text field, a segmented control or a tool
group), it rounds by half the height.

The states a control enters are listed under its table; a `StateValue` value
for another state never applies.

### ButtonStyle

`ButtonStyle::tengen()` is SwiftUI's `.bordered`:

| Field | Type | Tengen |
|---|---|---|
| `height`, `padding_x` | `f32` | `BUTTON.height`, `BUTTON.padding_x` |
| `shape` | `Shape` | `RoundedRectangle { radius: BUTTON.radius, style: Circular }` |
| `background` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, hovered `Role(ControlHover)`, disabled `Role(ControlBackground)` |
| `label` | `StateValue<ShapeStyle>` | `Role(Label)`, disabled `Role(TertiaryLabel)` |
| `border` | `Option<(ShapeStyle, f32)>` | `None`; paint and width, drawn inside |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `disabled_opacity` | `f32` | `1.0`, `1.0`: the disabled look is in the paints |
| `pressed_scale` | `f32` | `BUTTON.pressed_scale` |
| `effect` | `ColorEffect` | `system_effect` |
| `font` | `Font` | `BODY` with `BUTTON.font_size` and `BUTTON.font_weight`; the label is one line centered in `height`, so `line_height` has no effect |
| `animation` | `Animation` | `None`: the pressed scale jumps; `Ease` or `Spring` animates it |

States: hovered, pressed, focused, disabled.

The presets are functions of the style, so they go straight into `.style`,
`Theme::style` or `.control_style`, and change only their own fields:

| Preset | SwiftUI | Changes |
|---|---|---|
| `ButtonStyle::bordered_prominent` | `.borderedProminent` | `background` `Accent` (disabled `Role(ControlBackground)`), `label` `Role(OnAccent)` (disabled `Role(TertiaryLabel)`), `border` `None` |
| `ButtonStyle::glass` | `.glass` | `background` `Role(RaisedBackground)` (hovered `Role(ControlHover)`, disabled `Role(ControlBackground)`), `border` `(Role(CardBorder), HAIRLINE)`, `shape` `Capsule` |
| `ButtonStyle::glass_prominent` | `.glassProminent` | `bordered_prominent`, then the border and shape of `glass` |
| `ButtonStyle::borderless` | `.borderless` | `background` `Clear` (hovered `Role(ControlHover)`), `border` `None` |

`Button::size(ControlSize)` is an instance override of `height`, `padding_x`,
`font` and `shape`: `Small` takes the `BUTTON.small_*` tokens, `Regular` the
plain ones and `Large` `BUTTON.large_height` with a `Capsule`. A `Capsule`
stays a `Capsule` at every size. A button that is `.toggled(true)` draws its
style through `bordered_prominent`.

```rust
Button::new("Save").style(ButtonStyle::bordered_prominent)
Button::new("Delete").style(|s| ButtonStyle {
    background: StateValue::new(ShapeStyle::Role(Role::Destructive)),
    ..s.bordered_prominent()
})
```

### SwitchStyle

| Field | Type | Tengen |
|---|---|---|
| `width`, `height` | `f32` | `SWITCH.width`, `SWITCH.height` |
| `knob_size` | `(f32, f32)` | `(SWITCH.knob_width, SWITCH.knob_height)` |
| `inset` | `f32` | `SWITCH.inset`: the knob's gap to the ends; it travels `width` − knob width − 2 × `inset` |
| `label_gap` | `f32` | `SETTINGS.check.gap` |
| `track_shape`, `knob_shape` | `Shape` | `Capsule`, `Capsule` |
| `track`, `fill` | `StateValue<ShapeStyle>` | `Role(ControlBackground)` while off, `Accent` while on |
| `knob`, `label` | `StateValue<ShapeStyle>` | `Color(SWITCH.knob_color)`, `Role(Label)` |
| `track_border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))`, drawn only while off |
| `knob_border` | `Option<(ShapeStyle, f32)>` | `None` |
| `knob_shadows` | `Vec<ShadowStyle>` | `[KNOB_SHADOW]` |
| `opacity` | `f32` | `1.0` |
| `disabled_opacity` | `f32` | `SWITCH.disabled_opacity`, for the switch and its label |
| `effect` | `ColorEffect` | `system_effect` |
| `label_position` | `Option<Edge>` | `None`: the label is only accessible, as in a [form row](../docs/barrierkit/11-views.md); `Leading` and `Trailing` put it beside the switch, `Top` and `Bottom` above and below |
| `layout_direction` | `LayoutDirection` | `LeftToRight`; `RightToLeft` puts the knob of a switch that is on at the left and swaps `Leading` and `Trailing` |
| `label_font` | `Font` | `BODY` |
| `animation` | `Animation` | `Ease { duration_ms: SWITCH.transition_ms }`: the knob slides; `None` makes it jump |

States: hovered, pressed, focused, disabled. `Switch::tint(Tint)` is an
instance override of `fill`.

### CheckboxStyle

| Field | Type | Tengen |
|---|---|---|
| `box_size`, `mark_size`, `label_gap` | `f32` | `CHECKBOX.size`, `CHECKBOX.mark`, `SETTINGS.check.gap` |
| `box_shape` | `Shape` | `RoundedRectangle { radius: CHECKBOX.radius, style: Circular }` |
| `box_background`, `fill` | `StateValue<ShapeStyle>` | `Role(ControlBackground)` while unchecked, `Accent` while checked |
| `mark`, `label` | `StateValue<ShapeStyle>` | `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`) |
| `box_border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))`, drawn only while unchecked |
| `box_shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `disabled_opacity` | `f32` | `1.0`, `1.0`, for the box and the label |
| `effect` | `ColorEffect` | `system_effect` |
| `label_position` | `Option<Edge>` | `Some(Edge::Trailing)`; `None` hides the label, which stays accessible |
| `layout_direction` | `LayoutDirection` | `LeftToRight`; `RightToLeft` swaps `Leading` and `Trailing` |
| `label_font` | `Font` | `BODY` |

States: hovered, pressed, focused, disabled. `Checkbox::tint(Tint)` is an
instance override of `fill`.

The label before the control, as in a settings list:

```rust
Checkbox::new("Remember me").style(|s| CheckboxStyle { label_position: Some(Edge::Leading), ..s })
Switch::new("Wi-Fi").style(|s| SwitchStyle { label_position: Some(Edge::Leading), ..s })
```

### SegmentedControlStyle

| Field | Type | Tengen |
|---|---|---|
| `padding`, `gap` | `f32` | `SEGMENTED.padding`, `SEGMENTED.gap` |
| `segment_height`, `segment_padding_x` | `f32` | `SEGMENTED.segment_height`, `SEGMENTED.segment_padding_x` |
| `shape`, `segment_shape` | `Shape` | `RoundedRectangle` of `SEGMENTED.radius` and of `SEGMENTED.segment_radius`, both `Circular` |
| `background` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, behind the segments |
| `segment`, `selected` | `StateValue<ShapeStyle>` | `Clear` for a segment that is not chosen, `Role(SelectedControl)` for the chosen one |
| `label`, `selected_label` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(Label)` |
| `border` | `Option<(ShapeStyle, f32)>` | `None`, around the group |
| `selected_shadows` | `Vec<ShadowStyle>` | `[KNOB_SHADOW]` |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `axis` | `Axis` | `Horizontal`: the segments share the width; `Vertical` stacks them at full width |
| `font` | `Font` | `BODY` with `SEGMENTED.font_size` and `SEGMENTED.font_weight`; `line_height` has no effect |

States: each segment is hovered, pressed and focused on its own; the group
has only `normal`. The arrow keys move the choice on both axes.

### ToolGroupStyle

| Field | Type | Tengen |
|---|---|---|
| `item_size` | `f32` | `TOOLGROUP.height`: the square slot of each item and the thickness of the group |
| `padding` | `f32` | `TOOLGROUP.padding_x`, at both ends along the axis |
| `separator_size` | `(f32, f32)` | `(TOOLGROUP.separator_w, TOOLGROUP.separator_h)`: along the axis, across it |
| `shape` | `Shape` | `Capsule` |
| `background` | `StateValue<ShapeStyle>` | Island sidebar: `Color` of the resolved tool group background, or `Role(ControlBackground)` inside a tinted `Backdrop`; Flat: `Clear` |
| `separator` | `Option<StateValue<ShapeStyle>>` | Island: `Color` of the tool group separator, or `Role(Separator)` inside a tinted `Backdrop`; Flat: `None`, no separators |
| `border` | `Option<(ShapeStyle, f32)>` | Island: the tool group rim, or `Role(CardBorder)`, `HAIRLINE` wide; Flat: `None` |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `axis` | `Axis` | `Horizontal`; `Vertical` stacks the items |

States: only `normal`. Tengen follows the user's sidebar style and reduce
transparency, which picks the opaque tool group colors.

### TextFieldStyle, SearchFieldStyle and SecureFieldStyle

Freya's `Input` draws `TextField` and `SearchField`. It paints one color per
part: a `Gradient` gives its first stop, `Backdrop` the first backdrop color
and a `Material` its tint, with the blur drawn by the kit. It draws the
border 1 wide, and 2 wide while focused.

| Field | Type | `TextFieldStyle` | `SearchFieldStyle` | `SecureFieldStyle` |
|---|---|---|---|---|
| `height`, `padding_x` | `f32` | `TEXT_FIELD.height`, `TEXT_FIELD.padding_x` | `TEXT_FIELD.height`, `SETTINGS.search_padding_x` | `TEXT_FIELD.height`, `TEXT_FIELD.padding_x` |
| `icon_size`, `icon_gap` | `f32` | — | `SETTINGS.search_icon`, `SETTINGS.search_gap` | — |
| `shape` | `Shape` | `RoundedRectangle { radius: TEXT_FIELD.radius, style: Circular }` | `Capsule` | as `TextFieldStyle` |
| `background` | `StateValue<ShapeStyle>` | `Role(ContentBackground)` | `Role(ControlBackground)` | `Role(ContentBackground)` |
| `border` | `StateValue<ShapeStyle>`; `Option<(ShapeStyle, f32)>` on `SecureFieldStyle` | `Role(Separator)`, focused `Accent` | `Clear`, focused `Accent` | `Some((Role(Separator), HAIRLINE))` |
| `text` | `StateValue<ShapeStyle>` | `Role(Label)` | `Role(Label)` | `Role(Label)`, the `•` |
| `placeholder` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)` | `Role(SecondaryLabel)` | — |
| `misspelling` | `StateValue<ShapeStyle>` | `Role(Destructive)`, the spelling squiggle | — | — |
| `icon`, `clear_icon` | `StateValue<ShapeStyle>` | — | `Role(TertiaryLabel)` magnifying glass, `Role(SecondaryLabel)` clear button | — |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` | the same | the same |
| `font` | `Font` | `BODY`; the text sits (`height` − `line_height`) / 2 from the top | the same | the same |

States: `TextField` and `SearchField` are hovered and focused;
`SecureField` is hovered, pressed and focused and draws the kit's focus ring.
The token capsules of a `SearchField` draw from the
[`TokenFieldStyle`](#tokenfieldstyle) in context, and the field of a
[`ComboBox`](#comboboxstyle) from the `TextFieldStyle` in context.

### LevelSliderStyle

| Field | Type | Tengen |
|---|---|---|
| `height` | `f32` | `SLIDER.height`; the slider fills its width |
| `track_height` | `f32` | `SLIDER.track` |
| `knob_size` | `Option<(f32, f32)>` | `Some((SLIDER.knob_width, SLIDER.knob_height))`; `None` draws no knob and the fill starts and ends at the edges |
| `fill_min` | `f32` | `0.0`: the shortest fill, so the symbol always sits on it |
| `symbol` | `Option<Symbol>` | `None`; a symbol drawn inside the track at the leading end |
| `icon_size`, `icon_inset` | `f32` | `SYMBOL.grid`, `SLIDER_FAT.lead_icon_x` |
| `track_shape`, `knob_shape` | `Shape` | `Capsule`, `Capsule`; the fill takes the track's shape |
| `track`, `fill`, `knob`, `icon` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, `Accent`, `Color(SWITCH.knob_color)`, `Role(OnAccent)` |
| `knob_border` | `Option<(ShapeStyle, f32)>` | `None` |
| `knob_shadows` | `Vec<ShadowStyle>` | `[KNOB_SHADOW]` |
| `opacity` | `f32` | `1.0`, for every part |
| `disabled_opacity` | `f32` | `SWITCH.disabled_opacity`, for the track, fill and symbol |
| `effect` | `ColorEffect` | `system_effect` |
| `layout_direction` | `LayoutDirection` | `LeftToRight`; `RightToLeft` fills from the right and swaps ← → |
| `animation` | `Animation` | `None`: the knob and the fill jump, as in Tengen |

States: hovered, pressed, focused, disabled. `LevelSlider::tint(Tint)` is an
instance override of `fill`. The preset `LevelSliderStyle::fat(symbol)` is the
Control Center bar: `height` and `track_height` `SLIDER_FAT.height`,
`knob_size` `None`, `fill_min` `SLIDER_FAT.fill_min` and the symbol.

```rust
LevelSlider::new("Volume").style(|s| s.fat(Symbol::SpeakerWave))
```

### RadioGroupStyle

| Field | Type | Tengen |
|---|---|---|
| `button_size`, `dot_size` | `f32` | `SETTINGS.radio_size`, `SETTINGS.radio_dot` |
| `label_gap`, `spacing` | `f32` | `SETTINGS.radio_gap` between a button and its label, `SETTINGS.radio_spacing` between options |
| `button_shape`, `dot_shape` | `Shape` | `Circle`, `Circle` |
| `button_background`, `fill` | `StateValue<ShapeStyle>` | `Role(ControlBackground)` while not chosen, `Accent` while chosen |
| `dot`, `label` | `StateValue<ShapeStyle>` | `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`) |
| `button_border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))`, drawn only while not chosen |
| `button_shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `disabled_opacity` | `f32` | `1.0`, `1.0`, for the button and the label |
| `effect` | `ColorEffect` | `system_effect` |
| `axis` | `Axis` | `Vertical`: a column of options; `Horizontal` puts them in a row |
| `label_position` | `Option<Edge>` | `Some(Edge::Trailing)`; `None` hides the labels, which stay accessible |
| `layout_direction` | `LayoutDirection` | `LeftToRight`; `RightToLeft` swaps `Leading` and `Trailing` |
| `label_font` | `Font` | `BODY` |

States: each option is hovered, pressed, focused and disabled on its own.
`RadioGroup::tint(Tint)` is an instance override of `fill`. The arrow keys
move the choice on both axes.

```rust
RadioGroup::new("Size").option("Small").option("Large").style(|s| RadioGroupStyle {
    axis: Axis::Horizontal,
    label_position: Some(Edge::Leading),
    ..s
})
```

### ColorWellStyle and ChoiceCardsStyle

The colors of the swatches and the previews of the cards are content: no
paint or `effect` changes them.

| Field | Type | `ColorWellStyle` | `ChoiceCardsStyle` |
|---|---|---|---|
| `swatch_size`; `card_size` | `f32`; `(f32, f32)` | `SETTINGS.swatch_size` | `(SETTINGS.card_width, SETTINGS.card_height)` |
| `column_width` | `f32` | `SETTINGS.swatch_column`, the width of a swatch with its name | — |
| `name_gap`; `caption_gap` | `f32` | `SETTINGS.swatch_name_gap` | `SETTINGS.card_caption_gap` |
| `spacing` | `f32` | `SETTINGS.swatch_gap` | `SETTINGS.card_gap`; also between wrapped lines |
| `swatch_shape`; `card_shape` | `Shape` | `Circle` | `RoundedRectangle { radius: SETTINGS.card_radius, style: Circular }` |
| `selection` | `StateValue<ShapeStyle>` | `Role(Label)`, the ring of the chosen swatch | `Accent`, the ring of the chosen card |
| `name`; `caption`, `selected_caption` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)`, shown only under the chosen swatch | `Role(SecondaryLabel)`, `Role(Label)` |
| `swatch_border`; `card_border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))` | `Some((Color(SETTINGS.card_outline), HAIRLINE))` |
| `swatch_shadows`; `card_shadows` | `Vec<ShadowStyle>` | empty | empty |
| `opacity`, `disabled_opacity` | `f32` | `1.0`, `SWITCH.disabled_opacity` | the same |
| `effect` | `ColorEffect` | `system_effect` | the same |
| `axis` | `Axis` | `Horizontal`; `Vertical` stacks them | the same |
| `wraps` | `bool` | `false`; `true` starts a new line when the next one does not fit | the same |
| `name_font`; `caption_font`, `selected_caption_font` | `Font` | `CAPTION` | `SETTINGS.card_caption_font`, the same with `SETTINGS.card_caption_selected_weight` |

States: each swatch and card is hovered, pressed, focused and disabled on its
own.

### IconButtonStyle

| Field | Type | Tengen |
|---|---|---|
| `size`, `icon_size` | `f32` | `ICON_BUTTON.size`, in a toolbar `TOOLGROUP.height`; `ICON_BUTTON.icon` |
| `shape` | `Shape` | `RoundedRectangle { radius: ICON_BUTTON.radius, style: Circular }`, in a toolbar `Capsule` |
| `background` | `StateValue<ShapeStyle>` | `Clear`, hovered `Role(ControlHover)`, disabled `Clear` |
| `icon` | `StateValue<ShapeStyle>` | `Role(Label)`, disabled `Role(TertiaryLabel)` |
| `border` | `Option<(ShapeStyle, f32)>` | `None` |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `disabled_opacity` | `f32` | `1.0`, `1.0` |
| `pressed_scale` | `f32` | `BUTTON.pressed_scale` |
| `effect` | `ColorEffect` | `system_effect` |
| `animation` | `Animation` | `None`: the pressed scale jumps |

States: hovered, pressed, focused, disabled. `ToolbarOverflow` is an
`IconButton`: its `.style` takes an `IconButtonStyle`.

### StepperStyle

| Field | Type | Tengen |
|---|---|---|
| `arrows_size` | `(f32, f32)` | `(STEPPER.width, STEPPER.height)` |
| `chevron_size`, `separator_height`, `value_gap` | `f32` | `STEPPER.chevron`, `HAIRLINE`, `STEPPER.gap` |
| `shape` | `Shape` | `RoundedRectangle { radius: STEPPER.radius, style: Circular }`, the arrows |
| `background`, `separator` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, `Role(Separator)` |
| `chevron`, `value` | `StateValue<ShapeStyle>` | `Role(Label)` (disabled `Role(TertiaryLabel)`), `Role(Label)` |
| `border` | `Option<(ShapeStyle, f32)>` | `None` |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `value_position` | `Option<Edge>` | `Some(Edge::Leading)`; `None` hides the value, which stays accessible |
| `layout_direction` | `LayoutDirection` | `LeftToRight`; `RightToLeft` swaps `Leading` and `Trailing` |
| `value_font` | `Font` | `BODY` |

States: the stepper is hovered, pressed and focused; each arrow is disabled at
its limit. The arrows of a `DatePicker` draw from the `StepperStyle` in
context, with only `normal` and that disabled state.

### ToggleGroupStyle

| Field | Type | Tengen |
|---|---|---|
| `padding`, `gap` | `f32` | `SEGMENTED.padding`, `SEGMENTED.gap` |
| `item_size` | `(f32, f32)` | `(ICON_BUTTON.size, SEGMENTED.segment_height)` |
| `shape`, `item_shape` | `Shape` | `RoundedRectangle` of `SEGMENTED.radius` and of `SEGMENTED.segment_radius`, both `Circular` |
| `background` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, behind the items |
| `item`, `selected` | `StateValue<ShapeStyle>` | `Clear` for a toggle that is off, `Accent` for one that is on |
| `label`, `selected_label` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(OnAccent)` |
| `border` | `Option<(ShapeStyle, f32)>` | `None`, around the group |
| `selected_shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `axis` | `Axis` | `Horizontal`; `Vertical` stacks the toggles |
| `font`, `bold_font` | `Font` | `BODY` with `SEGMENTED.font_weight`; `BODY` with `LARGE_TITLE.weight` for `Face::Bold`; `line_height` has no effect |

States: each toggle is hovered, pressed and focused on its own; the group has
only `normal`.

### ToolbarSearchStyle

| Field | Type | Tengen |
|---|---|---|
| `width` | `f32` | `TOOLBAR_SEARCH_WIDTH`, the width of the open field |

The open field is a `SearchField` and draws from the `SearchFieldStyle` in
context; the collapsed button is an `IconButton` and draws from the
`IconButtonStyle` in context. Change them with `.control_style` around the
toolbar search:

```rust
ToolbarSearch::new(query, "Search")
    .control_style(|s: SearchFieldStyle| SearchFieldStyle { shape: Shape::Rectangle, ..s })
```

### MenuButtonStyle, ComboButtonStyle and HelpLinkStyle

| Field | Type | `MenuButtonStyle` | `ComboButtonStyle` | `HelpLinkStyle` |
|---|---|---|---|---|
| `height`; `size` | `f32` | `BUTTON.height` | `BUTTON.height` | `size` `BUTTON.height`, a square |
| `padding_x` | `f32` | `BUTTON.padding_x` | `BUTTON.padding_x`, the label half | — |
| `menu_padding_x` | `f32` | — | `BUTTON.menu_padding_x`, the chevron half | — |
| `chevron_gap` | `f32` | `SPACE_1` | — | — |
| `chevron_size` | `f32` | `SETTINGS.dropdown_chevron_glyph` | `SETTINGS.dropdown_chevron_glyph` | — |
| `separator_size` | `(f32, f32)` | — | `(HAIRLINE, BUTTON.height / 2)` | — |
| `shape` | `Shape` | `RoundedRectangle { radius: BUTTON.radius, style: Circular }` | the same | `Circle` |
| `background` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, hovered `Role(ControlHover)` | `Role(ControlBackground)`, behind both halves | `Role(ControlBackground)`, hovered `Role(ControlHover)` |
| `segment` | `StateValue<ShapeStyle>` | — | `Clear`, hovered `Role(ControlHover)`, for each half | — |
| `label` | `StateValue<ShapeStyle>` | `Role(Label)` | `Role(Label)` | `Role(Label)`, the `?` |
| `chevron` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)` | `Role(SecondaryLabel)` | — |
| `separator` | `StateValue<ShapeStyle>` | — | `Role(Separator)` | — |
| `border` | `Option<(ShapeStyle, f32)>` | `None` | `None` | `None` |
| `shadows` | `Vec<ShadowStyle>` | empty | empty | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` | the same | the same |
| `pressed_scale` | `f32` | — | — | `BUTTON.pressed_scale` |
| `font` | `Font` | `BODY` | `BODY` | `BODY` |
| `animation` | `Animation` | — | — | `None`: the pressed scale jumps |

States: hovered, pressed and focused; each half of a `ComboButton` on its own,
and its group only `normal`. `ShareLink` is a `MenuButton`: its `.style` takes
a `MenuButtonStyle`.

### ControlGroupStyle

| Field | Type | Tengen |
|---|---|---|
| `height` | `f32` | `BUTTON.height` |
| `separator_size` | `(f32, f32)` | `(HAIRLINE, BUTTON.height / 2)`, between children |
| `shape` | `Shape` | `RoundedRectangle { radius: BUTTON.radius, style: Circular }` |
| `background`, `separator` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, `Role(Separator)` |
| `border` | `Option<(ShapeStyle, f32)>` | `None` |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |

States: only `normal`. The children keep their own styles: a `Button` inside
draws from its `ButtonStyle`.

### PopUpButtonStyle

| Field | Type | Tengen |
|---|---|---|
| `height`, `gap` | `f32` | `SETTINGS.popup_height`, `SETTINGS.popup_text_gap` between the icon, the label and the indicator |
| `icon_size` | `f32` | `SYMBOL.grid`, the SVG icon of the chosen item |
| `indicator_size`, `chevron_size` | `f32` | `SETTINGS.popup_chevron`, `SETTINGS.dropdown_chevron_glyph` |
| `indicator_shape` | `Shape` | `Circle`; the focus ring takes its corners |
| `label`, `indicator`, `chevron` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(ControlBackground)`, `Role(Label)` |
| `indicator_border` | `Option<(ShapeStyle, f32)>` | `None` |
| `indicator_shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `font` | `Font` | `BODY` |

States: hovered, pressed, focused. The menu it opens is a [menu view](../docs/barrierkit/07-menus-and-popovers.md).

### ComboBoxStyle

The field is a text field: it draws from the `TextFieldStyle` in context, whose
`misspelling` has no effect and whose `opacity` dims the whole combo box.
`ComboBoxStyle` holds the arrow button.

| Field | Type | Tengen |
|---|---|---|
| `button_size` | `(f32, f32)` | `(COMBO_BOX.button_width, COMBO_BOX.button_height)` |
| `button_inset`, `text_gap` | `f32` | `COMBO_BOX.button_inset_x` from the trailing edge, `COMBO_BOX.text_gap` between the text and the button |
| `chevron_size` | `f32` | `COMBO_BOX.chevron` |
| `button_shape` | `Shape` | `RoundedRectangle { radius: COMBO_BOX.button_radius, style: Circular }` |
| `button`, `chevron` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, `Role(Label)` |
| `button_border` | `Option<(ShapeStyle, f32)>` | `None` |
| `button_shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect`, for the button |

States: the field is hovered and focused; the button has only `normal`.

```rust
ComboBox::new(shell, "Shell")
    .style(|s| ComboBoxStyle { button: StateValue::new(ShapeStyle::Clear), ..s })
    .control_style(|s: TextFieldStyle| TextFieldStyle { shape: Shape::Capsule, ..s })
```

### TokenFieldStyle

| Field | Type | Tengen |
|---|---|---|
| `height`, `padding_x` | `f32` | `TEXT_FIELD.height`, the least height and the height of each line, `TEXT_FIELD.padding_x` |
| `gap` | `f32` | `TOKEN_FIELD.gap`, between tokens and between wrapped lines |
| `token_height`, `token_padding_x` | `f32` | `TOKEN_FIELD.token_height`, `TOKEN_FIELD.token_padding_x` |
| `input_width` | `f32` | `TOKEN_FIELD.input_min_width`, the text input after the tokens |
| `shape`, `token_shape` | `Shape` | `RoundedRectangle { radius: TEXT_FIELD.radius, style: Circular }`, `Capsule` |
| `background`, `text`, `placeholder` | `StateValue<ShapeStyle>` | `Role(ContentBackground)`, `Role(Label)`, `Role(SecondaryLabel)` |
| `token`, `selected_token` | `StateValue<ShapeStyle>` | `Color` of the accent at `TOKEN_FIELD.token_opacity`, `Accent` |
| `token_label`, `selected_token_label` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(OnAccent)` |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))` |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `font` | `Font` | `BODY`, for the text and the tokens |

States: only `normal`; a token is selected with the arrows or a click.

### DatePickerStyle

| Field | Type | Tengen |
|---|---|---|
| `height`, `padding_x` | `f32` | `TEXT_FIELD.height`, `TEXT_FIELD.padding_x` |
| `stepper_gap` | `f32` | `STEPPER.gap`, between the field and the arrows |
| `calendar_size` | `(f32, f32)` | `(DATE_PICKER.calendar_width, DATE_PICKER.calendar_height)`, without the padding |
| `calendar_padding`, `header_height`, `weekday_height` | `f32` | `MENU_POPOVER.padding`, `DATE_PICKER.header_height`, `DATE_PICKER.weekday_height` |
| `chevron_size` | `f32` | `CAPTION.size`, the month arrows |
| `shape`, `segment_shape`, `day_shape` | `Shape` | `RoundedRectangle` of `TEXT_FIELD.radius`, `DATE_PICKER.segment_radius` and `DATE_PICKER.day_radius`, all `Circular` |
| `background`, `text`, `separator` | `StateValue<ShapeStyle>` | `Role(ContentBackground)`, `Role(Label)`, `Role(SecondaryLabel)` the `/` and `:` |
| `selected_segment`, `selected_text` | `StateValue<ShapeStyle>` | `Accent`, `Role(OnAccent)`: the chosen part while the field has focus |
| `title`, `chevron`, `weekday` | `StateValue<ShapeStyle>` | `Role(Label)` the month, `Role(SecondaryLabel)`, `Role(SecondaryLabel)` |
| `day`, `other_month_day` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(SecondaryLabel)` |
| `selected_day`, `selected_day_label` | `StateValue<ShapeStyle>` | `Accent`, `Role(OnAccent)` |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))` |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect`, for the field |
| `font`, `title_font`, `day_font` | `Font` | `BODY`, `HEADLINE`, `CAPTION` for the days and weekdays |

States: the field is hovered, pressed and focused; the calendar has only
`normal`, on the popover background. The arrows draw from the
[`StepperStyle`](#stepperstyle) in context.

### PathControlStyle

| Field | Type | Tengen |
|---|---|---|
| `height`, `gap` | `f32` | `PATH_CONTROL.height`, `PATH_CONTROL.gap` between parts and chevrons |
| `icon_size`, `icon_gap`, `chevron_size` | `f32` | `PATH_CONTROL.icon`, `PATH_CONTROL.icon_gap`, `PATH_CONTROL.chevron` |
| `icon`, `label`, `chevron` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)`, `Role(Label)`, `Role(TertiaryLabel)` |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `font` | `Font` | `BODY` |

States: only `normal`.

### ImageWellStyle

| Field | Type | Tengen |
|---|---|---|
| `size`, `inset` | `f32` | `IMAGE_WELL.size`, a square; `IMAGE_WELL.inset` around the image, whose corners are the well's minus `inset` |
| `shape` | `Shape` | `RoundedRectangle { radius: IMAGE_WELL.radius, style: Circular }` |
| `background` | `StateValue<ShapeStyle>` | `Role(ContentBackground)` |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))` |
| `drop_border` | `(ShapeStyle, f32)` | `(Accent, IMAGE_WELL.drop_border)`, in place of `border` while an image is dragged over it |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |

States: hovered, pressed, focused. The image is content.

### ColorPickerStyle

The color in the well and in the panel is content: no paint or `effect`
changes it. The hex text keeps its contrast with the color.

| Field | Type | Tengen |
|---|---|---|
| `size` | `(f32, f32)` | `(BUTTON.height × 2, BUTTON.height)`; the panel's preview takes its height |
| `panel_size` | `(f32, f32)` | `(SPLIT_VIEW.inspector, BODY.line_height × 7)` |
| `spacing` | `f32` | `SPACE_2`, between the rows of the panel |
| `shape` | `Shape` | `RoundedRectangle { radius: BUTTON.radius, style: Circular }`, the well and the preview |
| `caption` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)`, the names of the sliders |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(CardBorder), HAIRLINE))` |
| `shadows` | `Vec<ShadowStyle>` | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `font` | `Font` | `BODY` |

States: the well is hovered, pressed and focused. The sliders of the panel
are `LevelSlider`s with Tengen's `LevelSliderStyle`.

`FontPicker` is a `Button`: its `.style` takes a `ButtonStyle`. The font
panel it opens has no style.

### ProgressViewStyle, ProgressRingStyle and SpinnerStyle

An SVG part (the ring and the spinner) takes one color: a `Gradient` gives its
first stop and a `Material` its tint.

| Field | Type | `ProgressViewStyle` | `ProgressRingStyle` | `SpinnerStyle` |
|---|---|---|---|---|
| `height`; `size` | `f32` | `PROGRESS.height`; the bar fills its width | `PROGRESS.ring_size` | `PROGRESS.spinner_size` |
| `indeterminate_length` | `f32` | `PROGRESS.indeterminate_segment`, the share of the track the sweeping segment takes | — | — |
| `stroke` | `f32` | — | `PROGRESS.ring_stroke` | `SYMBOL.stroke` |
| `spokes` | `u32` | — | — | `PROGRESS.spinner_spokes` |
| `shape` | `Shape` | `Capsule`, the track and the fill | — | — |
| `track`, `fill`; `spoke` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, `Accent` | `Role(ControlBackground)`, `Accent` | `spoke` `Role(SecondaryLabel)`, fading around the turn |
| `border` | `Option<(ShapeStyle, f32)>` | `None`, around the track | — | — |
| `shadows` | `Vec<ShadowStyle>` | empty | — | — |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` | the same | the same |
| `animation` | `Animation` | `Custom { duration_ms: PROGRESS.indeterminate_ms, curve }` with an ease-in-out cubic curve: the segment sweeps there and back | — | `Custom { duration_ms: PROGRESS.spinner_ms, curve: identity }`: one turn, spoke by spoke, again and again |

States: only `normal`. `ProgressView::tint(Tint)` is an instance override of
`fill`. With `Animation::None` (also with reduce motion) the segment rests at
the start and the spinner stands still.

### CapacityIndicatorStyle

| Field | Type | Tengen |
|---|---|---|
| `height`, `segment_gap` | `f32` | `LEVEL_INDICATOR.height`, `LEVEL_INDICATOR.segment_gap` between the cells of `discrete` |
| `shape` | `Shape` | `RoundedRectangle { radius: LEVEL_INDICATOR.radius, style: Circular }`, the bar or each cell |
| `track` | `StateValue<ShapeStyle>` | `Role(ControlBackground)` |
| `fill`, `warning`, `critical` | `StateValue<ShapeStyle>` | `Color` of `LEVEL_INDICATOR_COLORS[0]`, `[1]` and `[2]`, picked by `level_color` |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(CardBorder), HAIRLINE))` |
| `shadows` | `Vec<ShadowStyle>` | empty, for the bar or each cell |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |

States: only `normal`.

### RelevancyIndicatorStyle, RatingIndicatorStyle and GaugeStyle

| Field | Type | `RelevancyIndicatorStyle` | `RatingIndicatorStyle` | `GaugeStyle` |
|---|---|---|---|---|
| `height`; `size` | `f32` | `LEVEL_INDICATOR.relevancy_height` | — | `GAUGE.size` |
| `line_width`, `gap` | `f32` | `LEVEL_INDICATOR.relevancy_line`, `LEVEL_INDICATOR.relevancy_gap` | — | — |
| `star_size` | `f32` | — | `LEVEL_INDICATOR.star` | — |
| `stroke`, `arc`, `marker_size` | `f32` | — | — | `GAUGE.stroke`, `GAUGE.arc` in degrees, `GAUGE.marker` |
| Paint | `StateValue<ShapeStyle>` | `line` `Role(SecondaryLabel)` | `star`, `empty_star` `Role(SecondaryLabel)` | `track` `Role(TertiaryLabel)`, `marker` `Role(Label)`, `capacity_track` `Role(ControlBackground)`, `capacity_fill` `Accent`, `label` `Role(Label)` |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` | the same | the same |
| `font` | `Font` | — | — | `CAPTION`, the value in the center |

States: `RatingIndicator` is hovered, pressed and focused; the others only
`normal`. The ring and the marker of a `Gauge` take one color each, like the
progress ring. `gauge_marker(style, value)` places the marker for a
`GaugeStyle`.

### MediaButtonStyle and RoundButtonStyle

The media buttons sit on artwork, so their Tengen paints are fixed colors.

| Field | Type | `MediaButtonStyle` | `RoundButtonStyle` |
|---|---|---|---|
| `height`, `padding_x`; `size` | `f32` | `MEDIA.button_height`, `MEDIA.button_padding_x` | `size` `MEDIA.round`, a square |
| `icon_size` | `f32` | `MEDIA.button_font_size` | `MEDIA.round_icon` |
| `icon_gap` | `f32` | `MEDIA.button_font_size` / 2 | — |
| `shape` | `Shape` | `RoundedRectangle { radius: MEDIA.button_radius, style: Circular }` | `Circle` |
| `background` | `StateValue<ShapeStyle>` | `Color(MEDIA.button_fill)` | `Color(MEDIA.round_glass)` |
| `selected` | `StateValue<ShapeStyle>` | — | `Color(MEDIA.button_fill)`, while `.selected(true)` |
| `label`; `icon`, `selected_icon` | `StateValue<ShapeStyle>` | `Color(MEDIA.button_ink)`, the text and the symbol | `Color(MEDIA.button_fill)`, `Color(MEDIA.button_ink)` |
| `border` | `Option<(ShapeStyle, f32)>` | `None` | `Some((Color(MEDIA.round_rim), HAIRLINE))` |
| `shadows` | `Vec<ShadowStyle>` | empty | empty |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` | the same |
| `pressed_scale` | `f32` | `BUTTON.pressed_scale` | `BUTTON.pressed_scale` |
| `font` | `Font` | `BODY` with `MEDIA.button_font_size` and `MEDIA.button_font_weight`; `line_height` has no effect | — |
| `animation` | `Animation` | `None`: the pressed scale jumps | `None` |

States: hovered, pressed, focused.

### LinkStyle

| Field | Type | Tengen |
|---|---|---|
| `label` | `StateValue<ShapeStyle>` | `Color` of the resolved link color, which keeps 4.5:1 on the window |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `font` | `Font` | `BODY` |

States: hovered, pressed, focused.

```rust
Link::new("Learn more…").style(|s| LinkStyle {
    label: StateValue::new(ShapeStyle::Accent).hovered(ShapeStyle::Role(Role::Label)),
    ..s
})
```

### TextStyle, BadgeStyle, DividerStyle, AvatarStyle and ImageStyle

These views wrap `label()` or `rect()` and keep Freya's methods. They resolve
their style when they are built, against the theme and the `.control_style`
of the components around the one that builds them: a `.control_style` on a
stack does not reach the views built in the same function. `.style` builds
the view again, so it comes right after `new`, before Freya's methods.

| Field | Type | `TextStyle` | `BadgeStyle` | `DividerStyle` | `AvatarStyle` | `ImageStyle` |
|---|---|---|---|---|---|---|
| Size | `f32` | — | `height`, `padding_x`: `BADGE.height`, `BADGE.padding_x`; the badge is at least as wide as it is tall | `thickness`: `HAIRLINE`; the line fills the width | `size`: `SETTINGS.account_avatar`, a square | `size`: `BODY.size`, a square |
| `shape` | `Shape` | — | `Capsule` | — | `Circle` | — |
| Paint | `StateValue<ShapeStyle>` | `foreground` `Role(Label)` | `background` `Role(Destructive)`, `label` `Role(OnAccent)` | `line` `Role(Separator)` | `background` `Role(SecondaryLabel)`, `label` `Color` of the ink that contrasts with the secondary label | `foreground` `Role(Label)`, the symbol |
| `border` | `Option<(ShapeStyle, f32)>` | — | `None` | — | `None` | — |
| `shadows` | `Vec<ShadowStyle>` | — | empty | — | empty | — |
| `effect` | `ColorEffect` | `system_effect` | the same | the same | the same | the same |
| Font | `Font` | `font` `BODY` | `font` `CAPTION` | — | `font` `HEADLINE` | — |

States: only `normal`. `Text::font(Font)` changes the font of one text after
its style. `Image::size` and `Image::color` are instance overrides of `size`
and of `foreground` (`Color`). `VStack`, `HStack`, `Scrim` and `Canvas` have
no style: their look is the [paint on views](#paint-on-views) and Freya's
methods.

```rust
Theme::tengen()
    .style(|s: TextStyle| TextStyle { font: CALLOUT, ..s })
    .style(|s: BadgeStyle| BadgeStyle {
        background: StateValue::new(ShapeStyle::Accent),
        ..s
    })
```

### MaterialStyle, TopGlassStyle and GlassContainerStyle

| Field | Type | `MaterialStyle` | `TopGlassStyle` | `GlassContainerStyle` |
|---|---|---|---|---|
| `tint` | `StateValue<ShapeStyle>` | `Color(MEDIA.material_tint)` | `Color` of the glass of the window: `raised_background` over a `Backdrop`, the window surface without one | `Color(MEDIA.round_glass)` |
| `solid` | `StateValue<ShapeStyle>` | — | — | `Color(MEDIA.material_tint)`, in place of `tint` when the user reduces transparency |
| `rim` | `StateValue<ShapeStyle>` | — | — | `Color(MEDIA.round_rim)`, one pixel along the outline |
| `blur` | `f32` | `MEDIA.material_blur` | — | `MEDIA.material_blur` |
| `glass` | `FullscreenGlassMetrics` | — | `FULLSCREEN_GLASS`: the height, blur, layers and the stops of the tint and the mask | — |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Color(MEDIA.material_rim), HAIRLINE))`, drawn inside | — | — |
| `effect` | `ColorEffect` | `system_effect` | the same | the same |
| `animation` | `Animation` | — | — | `Ease { duration_ms: DURATION_BASE }`: the shapes morph; `None` makes them jump |

States: only `normal`. When the user reduces transparency the material and
the top glass become their `tint`, opaque, and the glass container its
`solid`. The corner radius of `Material::new` and the radii of
`GlassContainer::item` stay arguments: they follow the content.

### BackdropStyle

The colors of a `Backdrop` come from its source: they are content. The
palette it gives its content has no style.

| Field | Type | Tengen |
|---|---|---|
| `angles` | `[f32; 3]` | `BACKDROP.angles`, the direction of each gradient layer in degrees |
| `opacity` | `f32` | `BACKDROP.opacity`, of each layer where it starts |
| `animation` | `Animation` | `Custom { duration_ms: DURATION_SLOW_MS, curve }` with an ease-in-out cubic curve: a new source fades in; `None` makes it jump |

### ScrollViewStyle

| Field | Type | Tengen |
|---|---|---|
| `thumb_width`, `track_width` | `f32` | `SCROLL_BAR.thumb` for the bar that shows while scrolling, `SCROLL_BAR.track` for the thumb and track of the bar that is always shown |
| `min_thumb_length`, `inset` | `f32` | `SCROLL_BAR.min_length`, `SCROLL_BAR.inset` from the edge |
| `edge_height` | `f32` | `SCROLL_BAR.edge_height`, the fade of `.edge_effect(true)` |
| `thumb_shape` | `Shape` | `Capsule`, the thumb and the track |
| `thumb`, `track` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(Label)`: the color of the paint with the opacity below |
| `edge` | `StateValue<ShapeStyle>` | `Role(WindowBackground)`, fading to clear |
| `top_line` | `StateValue<ShapeStyle>` | `Color` of the sidebar line, the line of `.top_line(true)` |
| `thumb_opacity`, `always_thumb_opacity`, `track_opacity` | `f32` | `SCROLL_BAR.opacity`, `SCROLL_BAR.always_opacity`, `SCROLL_BAR.track_opacity` |
| `effect` | `ColorEffect` | `system_effect` |
| `hide_delay_ms` | `u64` | `SCROLL_BAR.hide_delay_ms`, from the last scroll until the bar fades out |
| `animation` | `Animation` | `Ease { duration_ms: DURATION_FAST_MS }`: the bar fades; `None` makes it appear and vanish at once |

States: only `normal`. The user's setting decides whether the bar is always
shown; `.scroll_bar(false)` hides it.

### TextEditorStyle

`TextEditor` is drawn like a [`TextField`](#textfieldstyle-searchfieldstyle-and-securefieldstyle)
with several lines, but its border does not turn to the accent: the focus
ring marks the focus. The find bar holds a `SearchField`, two `IconButton`s
and a small `Button`, which take their own styles.

| Field | Type | Tengen |
|---|---|---|
| `padding_x`, `padding_y` | `f32` | `TEXT_FIELD.padding_x`; (`TEXT_FIELD.height` − `BODY.line_height`) / 2 |
| `spacing`, `find_bar_spacing` | `f32` | `SPACE_1` between the find bar and the text, `SPACE_2` inside the find bar |
| `shape` | `Shape` | `RoundedRectangle { radius: TEXT_FIELD.radius, style: Circular }`; a `Capsule` rounds by half of one line |
| `background` | `StateValue<ShapeStyle>` | `Role(ContentBackground)` |
| `text`, `placeholder` | `StateValue<ShapeStyle>` | `Role(Label)`, the caret too; `Role(TertiaryLabel)` |
| `link`, `misspelling`, `caption` | `StateValue<ShapeStyle>` | `Accent`, `Role(Destructive)`, `Role(SecondaryLabel)` for the match count |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))`, drawn inside |
| `opacity`, `effect` | `f32`, `ColorEffect` | `1.0`, `system_effect` |
| `font`, `caption_font` | `Font` | `BODY`, `CAPTION` |

States: hovered, focused.

### RuleEditorStyle

The rows hold `PopUpButton`s, `TextField`s and `IconButton`s, which take
their own styles.

| Field | Type | Tengen |
|---|---|---|
| `spacing`, `row_spacing` | `f32` | `SPACE_2` between the rows, `SPACE_1` inside a row |
| `indent` | `f32` | `DISCLOSURE.indent` per level of a nested group |
| `label` | `StateValue<ShapeStyle>` | `Role(Label)`, the text "of the following" |
| `effect` | `ColorEffect` | `system_effect` |
| `font` | `Font` | `BODY` |

States: only `normal`.

### PrintPreviewStyle

The pages are content: no paint or `effect` changes them.

| Field | Type | Tengen |
|---|---|---|
| `padding`, `spacing`, `caption_gap` | `f32` | `SPACE_3` around the pages, `SPACE_3` between them, `SPACE_2` between a page and its number |
| `page_shape` | `Shape` | `Rectangle` |
| `page`, `caption` | `StateValue<ShapeStyle>` | `Role(ContentBackground)` behind each page, `Role(SecondaryLabel)` for the page number |
| `page_shadows` | `Vec<ShadowStyle>` | one shadow in the separator color with `SPACE_2` of blur |
| `effect` | `ColorEffect` | `system_effect` |
| `caption_font` | `Font` | `CAPTION` |

States: only `normal`.

### ChartStyle

One style for `Chart`, `SectorChart`, `HeatMap`, `StreamChart` and
`Chart3D`, so a theme colors every chart alike. Each chart draws only the
fields of its parts. The height or size of a chart stays an option of the
chart (`.height`, `.size`): it is layout.

| Field | Type | Tengen | Drawn by |
|---|---|---|---|
| `axis_width`, `axis_gap` | `f32` | `CHART.y_axis_width`, `CHART.axis_gap` between an axis label and the plot | `Chart`; `Chart3D` makes its camera labels twice as wide |
| `spacing`, `group_spacing` | `f32` | `SPACE_1` between the plot, the labels, the legend entries, a swatch and its label and the readout rows; `SPACE_2` between the sections | `spacing` `Chart`, `SectorChart`, `StreamChart`; `group_spacing` `SectorChart`, `Chart3D` |
| `legend_gap`, `legend_swatch` | `f32` | `CHART.legend_gap` between the entries of a row, `CHART.legend_swatch` | `Chart`, `SectorChart`, `StreamChart` |
| `tooltip_padding` | `(f32, f32)` | `(TOOLTIP.padding_x, TOOLTIP.padding_y)` | `Chart`, the readout of a selected point |
| `line_width`, `point_size`, `rule_width`, `grid_width` | `f32` | `CHART.line_width`, `CHART.point_size`, `CHART.rule_width`, `CHART.grid_width` | `Chart`; `grid_width` and `rule_width` also the guides of `StreamChart`, `grid_width` and `point_size` the box and points of `Chart3D` |
| `bar_gap`, `bar_radius` | `f32` | `CHART.bar_gap`, the share of a category left empty; `CHART.bar_radius` | `Chart` |
| `sector_inner`, `sector_inset`, `sector_radius` | `f32` | `CHART.sector_inner`, the hole of a donut; `CHART.sector_inset` degrees between slices; `CHART.sector_radius` | `SectorChart` |
| `heat_gap`, `heat_radius` | `f32` | `CHART.heat_gap`, `CHART.heat_radius` | `HeatMap` |
| `stream_hole` | `f32` | `CHART.stream_hole`, the hole of the flow clock | `StreamChart` |
| `legend_shape`, `tooltip_shape` | `Shape` | `Circle`; `RoundedRectangle { radius: TOOLTIP.radius, style: Circular }` | the legends; `Chart` |
| `series` | `Vec<ShapeStyle>` | `Color` of each of `CHART_SERIES`, repeated when there are more series | every chart but `HeatMap`; `Chart3D` goes from the first to the third |
| `grid`, `rule`, `hover`, `halo` | `StateValue<ShapeStyle>` | `Role(Separator)`, `Role(SecondaryLabel)`, `Role(TertiaryLabel)`, `Role(ContentBackground)` around the hovered points | `Chart`; `grid` also `StreamChart`, `Chart3D` |
| `heat_empty`, `heat_fill` | `StateValue<ShapeStyle>` | `Role(ControlBackground)` for the lowest value, `Accent` for the highest | `HeatMap` |
| `axis_label`, `legend_label` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)`, `Role(Label)`; the center of a donut takes `legend_label` for the value and `axis_label` for the name | `axis_label` `Chart`, `SectorChart`, `Chart3D`; `legend_label` `Chart`, `SectorChart`, `StreamChart` |
| `tooltip_background`, `tooltip_label` | `StateValue<ShapeStyle>` | `Color` of the popover background and text | `Chart` |
| `tooltip_border` | `Option<(ShapeStyle, f32)>` | `Some((Color of the popover separator, HAIRLINE))` | `Chart` |
| `area_opacity`, `unselected_opacity`, `hover_opacity` | `f32` | `CHART.area_opacity`, `CHART.unselected_opacity` of the other slices, `CHART.hover_opacity` of the hovered row of horizontal bars | `Chart`; `SectorChart` |
| `effect` | `ColorEffect` | `system_effect` | every chart |
| `font`, `value_font`, `title_font` | `Font` | `CAPTION` for the labels, legend and readout; `HEADLINE` for the value in a donut; `CALLOUT` for the camera labels | `font` `Chart`, `SectorChart`, `StreamChart`; `value_font` `SectorChart`; `title_font` `Chart3D` |
| `animation` | `Animation` | `Ease { duration_ms: DURATION_BASE }`: new data blends in and a time axis slides; `None` makes it jump | `Chart` |

States: only `normal`. `Chart3D` keeps its camera controls, a
`SegmentedControl`, `LevelSlider`s and a small `Button`, which take their own
styles.

```rust
Theme::tengen().style(|s: ChartStyle| ChartStyle {
    series: vec![ShapeStyle::Accent, ShapeStyle::Role(Role::SecondaryLabel)],
    bar_radius: 4.0,
    ..s
})
```

### TitlebarStyle and FullscreenBarStyle

`Titlebar` has two bars: the centered one (no sidebar) and the one beside a
sidebar (`.with_sidebar(true)`). Its paints are `focused` while the window is
active.

| Field | Type | Tengen |
|---|---|---|
| `height`, `close_only_height` | `f32` | `TITLEBAR_CLIENT_HEIGHT`; `TITLEBAR.thickness` for `.close_only(true)`, utility and HUD windows |
| `sidebar_height`, `sidebar_padding_top` | `f32` | `SETTINGS.toolbar_height`, `SETTINGS.toolbar_padding_top` with an island sidebar; `TITLEBAR_CLIENT_HEIGHT` and `0.0` with a flat one. `sidebar_bar_metrics()` returns them |
| `padding_x`, `sidebar_padding_x` | `f32` | `TITLEBAR_PADDING_X`; `SETTINGS.toolbar_padding_x` (island) or `TITLEBAR_PADDING_X` (flat). With the traffic lights in the centered bar the side padding is half of `height` minus the lights' `diameter` |
| `spacing` | `f32` | `0.0`, between the bar's items |
| `background` | `StateValue<ShapeStyle>` | `Color` of `titlebar_inactive`, focused `Color` of the window surface; `Clear` over a `Backdrop` |
| `sidebar_background` | `StateValue<ShapeStyle>` | `Color` of the window surface while the sidebar shows; `Clear` over a `Backdrop` |
| `close_only_background` | `StateValue<ShapeStyle>` | `Clear` |
| `title` | `StateValue<ShapeStyle>` | `Color` of the unfocused title, focused `Color` of the focused one; over a `Backdrop` `Role(SecondaryLabel)`, focused `Role(Label)` |
| `hairline` | `Option<(ShapeStyle, f32)>` | `Some((Color of the sidebar line, HAIRLINE))`, at the bottom of the bar beside a sidebar |
| `effect` | `ColorEffect` | `system_effect` |
| `title_alignment`, `sidebar_title_alignment` | `Alignment` | `Center` between the two equal zones; `Start`, next to the left zone |
| `title_font`, `sidebar_title_font` | `Font` | `TITLEBAR.font`, `TITLE_2`; one line, so `line_height` has no effect |

States: focused (the window is active). In full screen the bar sits on the
full-screen glass and draws no background. `.spacing(px)` is an instance
override of `spacing`; `.hairline(false)` sets `hairline` to `None`.

`FullscreenBarStyle` paints the glass under the overlaid bar and the rail in
full screen. The kind of bar stays `WindowOptions::fullscreen_bar`. It has no
instance level: set it in the theme or with `.control_style` around the
window.

| Field | Type | Tengen |
|---|---|---|
| `band`, `glass` | `FullscreenGlassMetrics` | `FULLSCREEN_BAND` for `FullscreenBar::Glass`, `FULLSCREEN_GLASS` for `ProgressiveGlass` |
| `tint` | `StateValue<ShapeStyle>` | `Color` of the glass of the window, as `TopGlassStyle` |
| `line` | `StateValue<ShapeStyle>` | `Role(Separator)`, the line under the `Glass` band |
| `effect` | `ColorEffect` | `system_effect` |

States: only `normal`. When the user reduces transparency every bar is the
solid `tint`.

```rust
Titlebar::new().style(|s| TitlebarStyle {
    title_alignment: Alignment::Start,
    hairline: None,
    ..s
})
```

### TrafficLightsStyle

`fills`, `rims` and `glyphs` hold one paint per light, in the order close,
minimize, zoom.

| Field | Type | Tengen |
|---|---|---|
| `diameter`, `spacing` | `f32` | `TITLEBAR.buttons.diameter`, `TITLEBAR.buttons.spacing` |
| `glyph_size`, `rim_width` | `f32` | `TRAFFIC_GLYPHS.box_size`, `TRAFFIC_RIM_WIDTH` |
| `shape` | `Shape` | `RoundedRectangle { radius: TITLEBAR.buttons.corner_radius, style: Circular }` |
| `fills`, `rims`, `glyphs` | `[StateValue<ShapeStyle>; 3]` | the `Color`s of `ctappearance::traffic::visual` for the user's `traffic_lights`: normal and disabled are the gray idle light without a glyph, focused the lit light, hovered the lit light with its glyph, pressed the darkened light. `glyphs` is `Clear` where no glyph shows |
| `opacity` | `f32` | `1.0` |
| `effect` | `ColorEffect` | `system_effect` |
| `axis` | `Axis` | `Horizontal` |

States: hovered (the pointer is over the group), pressed (that light), focused
(the window is focused), disabled (zoom without zoom support, the other
lights with `.close_only(true)`). `.round(true)` is an instance override of
`shape` (`Circle`), `.vertical(true)` of `axis`. `Titlebar`, the rail and the
sidebar place their lights with `TrafficLights::for_window`, so their size
comes from the theme and the containers.

### WindowChromeStyle

`WindowChrome::new(child).style(..)` takes the layout and the look of the
chrome. Its paints are `focused` while the window is active.

| Field | Type | Tengen |
|---|---|---|
| `layout` | `TitlebarLayout` | `Top`; `Rail` for the vertical bar |
| `rail_style` | `RailStyle` | `Normal`; `Unified` puts the column inside the sidebar |
| `reveal` | `Reveal` | `Always` ([9](../docs/barrierkit/09-titlebars.md#reveal)) |
| `resting_edge` | `RestingEdge` | `None`; `Line` draws `line_edge`, `Accent` `accent_edge` |
| `round_lights` | `bool` | `false`; `true` makes the window's traffic lights round |
| `rail_width` | `f32` | `TITLEBAR_RAIL.width`, at least `TITLEBAR_RAIL.actions_width` with rail items |
| `rail_padding_top`, `rail_padding_bottom`, `rail_spacing` | `f32` | `TITLEBAR_RAIL.padding_top`, `TITLEBAR_RAIL.padding_bottom`, `TITLEBAR.buttons.spacing`; the unified column takes the last two |
| `pill_inset`, `pill_padding` | `f32`, `(f32, f32)` | `TITLEBAR_RAIL.pill_inset`, `(pill_padding_x, pill_padding_y)` of the `Popup` pill |
| `pill_shape` | `Shape` | `RoundedRectangle { radius: TITLEBAR_RAIL.pill_radius, style: Circular }` |
| `rail_background` | `StateValue<ShapeStyle>` | `Color` of `titlebar_inactive`, focused `Color` of the window surface; `Clear` with an island sidebar |
| `drawer_background` | `StateValue<ShapeStyle>` | `Color` of the window surface, the rail of `Reveal::Drawer` |
| `title` | `StateValue<ShapeStyle>` | `Color` of the unfocused title, focused `Color` of the focused one |
| `pill_background` | `StateValue<ShapeStyle>` | `Role(RaisedBackground)` |
| `rail_line` | `Option<(ShapeStyle, f32)>` | `Some((Color of the sidebar line, HAIRLINE))` on the right of the rail; `None` with an island sidebar |
| `line_edge`, `accent_edge` | `(ShapeStyle, f32)` | `(Color of the sidebar line, TITLEBAR_RAIL.edge_line)`, `(Accent, TITLEBAR_RAIL.edge_accent)` |
| `pill_border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))`, drawn inside |
| `shadows` | `Vec<ShadowStyle>` | `[SHADOW.contact]` (`SHADOW_DARK.contact` in dark) with its `opacity`, under a drawer and the pill |
| `effect` | `ColorEffect` | `system_effect` |
| `title_angle` | `f32` | `TITLEBAR_RAIL.title_angle`, the rotation of the rail title in degrees |
| `title_font` | `Font` | `TITLEBAR.font` at `TITLEBAR_RAIL.title_size`; one line |
| `animation` | `Animation` | `Ease { duration_ms: DURATION_BASE }`: a hidden bar slides and the lights fade in; `None` makes them jump |

States: focused (the window is active). In full screen the rail sits on the
full-screen glass and draws no background. A `Theme` that sets `layout` takes
effect only inside a `WindowChrome`.

```rust
WindowChrome::new(content).style(|s| WindowChromeStyle {
    layout: TitlebarLayout::Rail,
    reveal: Reveal::Drawer,
    resting_edge: RestingEdge::Accent,
    ..s
})
```

### SidebarStyle, SidebarItemStyle, SidebarSectionStyle, SidebarAccountStyle and SymbolTileStyle

`SidebarStyle` is not `ctappearance::SidebarStyle`, the user's island or flat
choice: the Tengen of each field follows that choice.

| Field | Type | Tengen |
|---|---|---|
| `padding_x`, `padding_top`, `padding_bottom` | `f32` | `SIDEBAR.padding_x`; `SETTINGS.sidebar_padding_top` without traffic lights; `SIDEBAR.padding_x` (island) or `0.0` (flat) |
| `lights_position`, `lights_gap` | `(f32, f32)`, `f32` | `(SIDEBAR.traffic_x, SIDEBAR.traffic_y)`; `SETTINGS.sidebar_lights_gap` between the lights and the first row |
| `shape` | `Shape` | `RoundedRectangle { radius: SIDEBAR.radius, style: Continuous }` (island), `Rectangle` (flat); a `Capsule` or `Circle` has no size to round by and stays square |
| `background` | `StateValue<ShapeStyle>` | `Color` of the island or flat surface; `Clear` over a `Backdrop` |
| `border`, `inner_border` | `Option<(ShapeStyle, f32)>` | island: `Color` of the rim with `SIDEBAR.rim_width` and of the inner line with `SIDEBAR.inner_width`, inside the rim; flat or over a `Backdrop`: `None` |
| `line` | `Option<(ShapeStyle, f32)>` | flat: `Some((Color of the sidebar line, SIDEBAR.inner_width))` on the right; island: `None` |
| `effect` | `ColorEffect` | `system_effect` |
| `animation` | `Animation` | `Ease { duration_ms: DURATION_BASE }`: the first row slides to the title bar row when the chrome switches to the `Rail`; `None` makes it jump |

`.backdrop(SidebarBackdrop::Tinted)` is an instance override of `background`
and `border` over a `Backdrop`.

| Field | Type | `SidebarItemStyle` | `SidebarSectionStyle` | `SidebarAccountStyle` | `SymbolTileStyle` |
|---|---|---|---|---|---|
| Size | `f32` | `height`, `padding_x`, `spacing`: `SIDEBAR.item_height`, `item_padding_x`, `item_gap` | `height`, `margin_top`, `padding_x`, `chevron_size`: `SIDEBAR.section_height`, `section_margin_top`, `section_padding_x`, `chevron` | `height`, `margin_bottom`, `padding_x`, `spacing`: `SETTINGS.account_height`, `account_margin_bottom`, `account_padding_x`, `account_gap` | `size`, `symbol_size`: `SIDEBAR.app_icon`, `SIDEBAR.item_icon` |
| `shape` | `Shape` | `RoundedRectangle { radius: SIDEBAR.item_radius, style: Circular }` | — | — | the same |
| Paint | `StateValue<ShapeStyle>` | `background` `Clear`, hovered `Color` of the hover; `selected_background` `Color` of the selection; `label`, `selected_label` `Color` of the item and selected ink. In an inactive window the inactive colors | `title`, `chevron`: `Color` of the section ink | `name`, `detail`: `Color` of the item ink and of the count ink | `background` `Accent`, `symbol` `Role(OnAccent)` |
| Border | `Option<(ShapeStyle, f32)>` | `selected_tile_border` `Some((Color of the selected icon rim, SIDEBAR.rim_width))`, the tile of a selected item | — | — | `border` `None`, drawn inside |
| `effect` | `ColorEffect` | `system_effect` | the same | the same | the same |
| Font | `Font` | `font`, `selected_font`: `BODY` at `SIDEBAR.item_text_size` with `SIDEBAR.item_weight` and `SIDEBAR.item_selected_weight`; one line | `font`: `BODY` at `SIDEBAR.section_size` with `SIDEBAR.section_weight` | `name_font`, `detail_font`: `SETTINGS.account_name_font`, `account_detail_font` | — |
| `animation` | `Animation` | — | `Ease { duration_ms: DURATION_BASE }`: the rows fold; `None` makes them jump | — | — |

States: `SidebarItem` and the header of `SidebarSection` take hovered,
pressed and focused; the rest only `normal`. `SidebarItem::emphasized(false)`
is an instance override of `selected_background` (the inactive selection) and
`selected_label` (`label`). `SymbolTile::new(symbol, tint)` overrides
`background` with `Color(tint)` and `.rim(rgba)` sets `border`; like `Badge`,
`.style` builds the tile again, so it comes right after `new`. The avatar of
`SidebarAccount` takes `AvatarStyle`.

```rust
Theme::tengen().style(|s: SidebarItemStyle| SidebarItemStyle {
    selected_background: StateValue::new(ShapeStyle::Role(Role::SelectedControl)),
    selected_label: StateValue::new(ShapeStyle::Role(Role::Label)),
    shape: Shape::Capsule,
    ..s
})
```

### SplitViewStyle, SplitPanesStyle and FloatingPaneStyle

The widths of a `SplitView` sidebar (`.sidebar_width`, `.sidebar_range`) stay
options: they are layout the user changes.

| Field | Type | Tengen |
|---|---|---|
| `inset` | `f32` | `SIDEBAR.inset` around an island sidebar, `0.0` around a flat one |
| `inspector_width` | `f32` | `SPLIT_VIEW.inspector` |
| `grip_width`, `grip_line_width`, `grip_line_inset` | `f32` | `SPLIT_VIEW.grip`, the area that resizes; `SPLIT_VIEW.grip_line`, `SPLIT_VIEW.grip_line_inset` from the top and the bottom |
| `grip_shape` | `Shape` | `Capsule` |
| `background` | `StateValue<ShapeStyle>` | `Color` of the window surface; `Clear` over a `Backdrop` |
| `grip` | `StateValue<ShapeStyle>` | `Clear`, hovered, pressed (dragging) and focused (keyboard) `Color` of the accent |
| `scrim` | `StateValue<ShapeStyle>` | `Role(Scrim)`, behind the sidebar over the content of a narrow window |
| `inspector_line` | `(ShapeStyle, f32)` | `(Role(Separator), HAIRLINE)` |
| `effect` | `ColorEffect` | `system_effect` |
| `animation` | `Animation` | `Ease { duration_ms: DURATION_BASE }`: the sidebar and the inspector slide in and out |
| `resize_animation` | `Animation` | `Ease { duration_ms: DURATION_FAST_MS }`: the sidebar follows a resize from the keyboard; a drag follows the pointer |

`SidebarToggle` is an `IconButton` and takes `IconButtonStyle`.

| Field | Type | `SplitPanesStyle` | `FloatingPaneStyle` |
|---|---|---|---|
| Size | `f32` | `divider_width`: `SPLIT_PANES.divider` | `title_height`, `outline_width`: `FLOATING_PANE.title_height`, `FLOATING_PANE.outline`, drawn outside |
| `shape` | `Shape` | — | `Rectangle`; the title bar takes its top corners |
| Paint | `StateValue<ShapeStyle>` | `divider` `Role(Separator)` | `background` `Role(WindowBackground)`, `title_bar` `Role(ControlBackground)`, `title` `Role(Label)`, `outline` `Role(Separator)`, focused `Accent` |
| `shadows` | `Vec<ShadowStyle>` | — | `[FLOATING_PANE.shadow]` with `shadow_y` and `shadow_blur` |
| `effect` | `ColorEffect` | `system_effect` | the same |
| `title_font` | `Font` | — | `CAPTION` |

States: `FloatingPane` is focused while `active`; `SplitPanes` only `normal`.
`SplitPanes::divider(rgba)` is an instance override of `divider`. The fields
`accent`, `title_bar` and `title_ink` of `FloatingPane` are instance overrides
of `outline` (focused), `title_bar` and `title`, applied before `.style`.

### FormStyle, GroupBoxStyle and DisclosureGroupStyle

`FormStyle` draws the whole `Form`: its sections and their rows. `Section`
and `Row` are data and take no style.

| Field | Type | Tengen |
|---|---|---|
| `padding`, `header_padding` | `[f32; 4]` | `LIST_GROUP.padding` around the form; `LIST_GROUP.header_padding` around a section title. Top, right, bottom, left. A footer takes the bottom, right and left of `header_padding` |
| `section_spacing` | `f32` | `SPACE_4` between sections |
| `row_min_height`, `row_padding`, `row_spacing` | `f32`, `(f32, f32)`, `f32` | `LIST_GROUP.row_min_height`; `(LIST_GROUP.row_padding_x, SPACE_2)`; `SPACE_3` between the labels and the accessory |
| `separator_inset` | `f32` | `LIST_GROUP.separator_inset`, the left inset of the line between rows; the line is a `Divider` and takes `DividerStyle` |
| `chevron_size` | `f32` | `CAPTION.size`, the chevron of a navigation row |
| `shape` | `Shape` | `RoundedRectangle { radius: LIST_GROUP.radius, style: Circular }`, the card of a section; a `Capsule` or `Circle` has no size to round by and stays square |
| `background` | `StateValue<ShapeStyle>` | `Role(ContentBackground)`, the card |
| `row_background` | `StateValue<ShapeStyle>` | `Clear`, hovered `Role(ControlHover)` |
| `highlight` | `StateValue<ShapeStyle>` | `Color` of the accent at `LIST_GROUP.highlight`, the row of `.highlighted(..)` |
| `header`, `footer`, `title`, `destructive_title`, `detail`, `value`, `chevron` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(SecondaryLabel)`, `Role(Label)`, `Role(Destructive)`, `Role(SecondaryLabel)`, `Role(SecondaryLabel)`, `Role(TertiaryLabel)` |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(CardBorder), HAIRLINE))`, drawn inside the card |
| `effect` | `ColorEffect` | `system_effect` |
| `header_font`, `footer_font`, `title_font`, `detail_font`, `value_font` | `Font` | `HEADLINE`, `CAPTION`, `BODY`, `CAPTION`, `BODY` |

States: the rows of `Row::navigation` and `Row::destructive` take hovered,
pressed and focused; the rest only `normal`. The width below which a row puts
its control under the label stays `LIST_GROUP.stack_below`: it is layout.

| Field | Type | `GroupBoxStyle` | `DisclosureGroupStyle` |
|---|---|---|---|
| Size | `f32` | `padding`, `spacing`, `label_inset`: `GROUP_BOX.padding`, `SPACE_1` between the label and the box, `GROUP_BOX.label_inset` | `row_height`, `indent`, `chevron_size`: `DISCLOSURE.row_height`, `DISCLOSURE.indent` of the chevron column and the children, `DISCLOSURE.chevron` |
| `shape` | `Shape` | `RoundedRectangle { radius: GROUP_BOX.radius, style: Circular }`; a `Capsule` or `Circle` stays square | — |
| Paint | `StateValue<ShapeStyle>` | `background` `Color` of the label color at `GROUP_BOX.fill_opacity`; `label` `Role(Label)` | `chevron` `Role(SecondaryLabel)`; `label` `Role(Label)` |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(CardBorder), HAIRLINE))`, drawn inside | — |
| `effect` | `ColorEffect` | `system_effect` | the same |
| `font` | `Font` | `BODY` | `BODY` |

States: the header of `DisclosureGroup` takes hovered, pressed and focused;
`GroupBox` only `normal`. The children open and close at once.

```rust
Form::new()
    .section(Section::titled("Network").row(Row::value("Status", "Connected")))
    .style(|s| FormStyle {
        shape: Shape::RoundedRectangle { radius: 16.0, style: CornerStyle::Continuous },
        background: StateValue::new(ShapeStyle::Material(Thickness::Thin)),
        ..s
    })
```

### TableStyle, CollectionViewStyle and OutlineViewStyle

`OutlineViewStyle` is one style for `OutlineView` and `Browser`, which draw
the same rows; the rows of the font panel of `FontPicker` take it too.

| Field | Type | `TableStyle` |
|---|---|---|
| `header_height`, `row_height`, `padding_x`, `header_spacing` | `f32` | `TABLE.header_height`, `TABLE.row_height`, `TABLE.padding_x` on each side of a cell, `SPACE_1` between a header title and its arrow |
| `header_label`, `sorted_header_label` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)`; `Role(Label)` for the sorted column, which also paints its arrow |
| `row_background`, `alternate_row_background`, `selected_row_background` | `StateValue<ShapeStyle>` | `Clear` for even rows; `Color` of the label color at `TABLE.zebra` for odd rows; `Accent` |
| `label`, `selected_label` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(OnAccent)` |
| `header_border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))`, drawn inside the header |
| `effect` | `ColorEffect` | `system_effect` |
| `header_font`, `font` | `Font` | `CAPTION`, its size also the arrow's; `BODY` |

States: a header cell takes hovered, pressed and focused; rows only `normal`.
The resize handle and the minimum column width stay `TABLE.resize_handle` and
`TABLE.min_column_width`: they are behavior.

| Field | Type | `CollectionViewStyle` |
|---|---|---|
| `tile_width`, `icon_size` | `f32` | `COLLECTION.tile`, `COLLECTION.icon` |
| `spacing`, `padding`, `label_spacing`, `label_padding_x` | `f32` | `COLLECTION.gap` between tiles, around a tile's content and on each side of the name; half of it between the icon and the name |
| `shape`, `label_shape` | `Shape` | `RoundedRectangle { radius: COLLECTION.radius, style: Circular }`, the tile and the icon area; `RoundedRectangle { radius: COLLECTION.gap, style: Circular }`, the name. A `Capsule` rounds the tile by half `tile_width`, the icon area by half `icon_size` and the name by half the `line_height` of `font` |
| `icon_background`, `selected_icon_background` | `StateValue<ShapeStyle>` | `Clear`; `Color` of the label color at `COLLECTION.selection` |
| `label_background`, `selected_label_background` | `StateValue<ShapeStyle>` | `Clear`; `Accent` |
| `label`, `selected_label` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(OnAccent)` |
| `effect` | `ColorEffect` | `system_effect` |
| `font` | `Font` | `CAPTION` |

States: tiles take hovered, pressed and focused.

| Field | Type | `OutlineViewStyle` |
|---|---|---|
| `row_height`, `indent`, `spacing`, `icon_size`, `chevron_size` | `f32` | `DISCLOSURE.row_height`; `DISCLOSURE.indent` per level and of the chevron column; `SPACE_1` between the parts of a row; `DISCLOSURE.icon`; `DISCLOSURE.chevron` |
| `column_width`, `column_padding_x` | `f32` | `DISCLOSURE.column_width`; `SPACE_1` on each side of a row. `Browser` only |
| `row_shape` | `Shape` | `RoundedRectangle { radius: DISCLOSURE.row_radius, style: Circular }` |
| `row_background`, `selected_row_background` | `StateValue<ShapeStyle>` | `Clear`, `Accent` |
| `label`, `selected_label`, `icon`, `selected_icon` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(OnAccent)`, `Accent`, `Role(OnAccent)` |
| `disclosure`, `column_chevron`, `selected_chevron` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)`, the disclosure chevron of `OutlineView`; `Role(TertiaryLabel)`, the chevron of a `Browser` row with children; `Role(OnAccent)` on a selected row |
| `column_separator` | `(ShapeStyle, f32)` | `(Role(Separator), HAIRLINE)`, between `Browser` columns |
| `effect` | `ColorEffect` | `system_effect` |
| `font` | `Font` | `BODY` |

States: only `normal`; selection has its own fields.

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

### TabViewStyle and WindowTabsStyle

| Field | Type | `TabViewStyle` |
|---|---|---|
| `padding`, `spacing`, `content_spacing` | `f32` | `TOOLBAR_TABS.padding` inside the strip, `TOOLBAR_TABS.gap` between tabs, `SPACE_2` between the strip and the page |
| `tab_min_width`, `tab_padding`, `icon_size`, `icon_spacing` | `f32`, `(f32, f32)`, `f32`, `f32` | `TOOLBAR_TABS.tab_min_width`; `(tab_padding_top, tab_padding_bottom)`; `TOOLBAR_TABS.icon`; `TOOLBAR_TABS.icon_gap` |
| `shape`, `tab_shape` | `Shape` | `RoundedRectangle` of `TOOLBAR_TABS.radius` and of `TOOLBAR_TABS.tab_radius`, `Circular`; a `Capsule` rounds by half the height of the strip or the tab |
| `background` | `StateValue<ShapeStyle>` | `Role(SunkenBackground)`, the strip |
| `tab_background`, `selected_tab_background` | `StateValue<ShapeStyle>` | `Clear`, hovered `Role(ControlBackground)`; `Role(ControlHover)` |
| `label`, `selected_label` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)`, `Accent`; the icon takes the same paint |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))`, drawn inside the strip |
| `effect` | `ColorEffect` | `system_effect` |
| `font` | `Font` | `CAPTION` at `TOOLBAR_TABS.font_size` with `TOOLBAR_TABS.font_weight`; one line, so `line_height` only sets the tab's height for a `Capsule` |

| Field | Type | `WindowTabsStyle` |
|---|---|---|
| `tab_height`, `padding`, `spacing`, `tab_padding_x` | `f32` | `BUTTON.height`; `SPACE_1` around the bar, between tabs and on each side of a tab's content. The bar is `tab_height` plus twice `padding` high |
| `symbol_size`, `new_tab_width` | `f32` | `ICON_BUTTON.icon`, the × and +; `ICON_BUTTON.size`, the + button |
| `tab_shape` | `Shape` | `Capsule` |
| `background` | `StateValue<ShapeStyle>` | `Role(ControlBackground)`, the bar |
| `tab_background`, `selected_tab_background` | `StateValue<ShapeStyle>` | `Clear`, hovered `Role(ControlHover)`; `Role(RaisedBackground)` |
| `label`, `symbol` | `StateValue<ShapeStyle>` | `Role(Label)`; `Role(SecondaryLabel)`, the × and + |
| `effect` | `ColorEffect` | `system_effect` |
| `font` | `Font` | `BODY` |

States: a tab of either view takes hovered, pressed and focused.

### TextViewStyle

| Field | Type | Tengen |
|---|---|---|
| `text`, `cursor` | `StateValue<ShapeStyle>` | `Role(Label)` |
| `effect` | `ColorEffect` | `system_effect` |

States: focused while the view edits. The `ink` of `TextView::new` and
`.cursor(color)` are instance overrides of `text` and `cursor`, applied
before `.style`; the font stays an argument of `new`, because it can be any
family. The selection keeps Freya's highlight.

### SheetStyle, AlertStyle and ContentUnavailableViewStyle

`Alert` draws on a sheet: it takes `SheetStyle` from the theme and the
containers, and `AlertStyle` for its own parts. Its buttons take
`ButtonStyle` and its checkbox `CheckboxStyle`.

| Field | Type | `SheetStyle` | `AlertStyle` | `ContentUnavailableViewStyle` |
|---|---|---|---|---|
| Size | `f32` | `padding`: `DIALOG_PADDING` | `width`, `spacing`, `buttons_spacing`, `buttons_margin_top`: `DIALOG_ALERT_WIDTH` with the sheet's `padding`, `SPACE_2`, `SPACE_2`, `SPACE_4` | `padding_y`, `spacing`, `symbol_size`: `SETTINGS.empty_padding_y`, `empty_gap`, `empty_icon` |
| `shape` | `Shape` | `RoundedRectangle { radius: DIALOG.corner_radius, style: Continuous }`; a `Capsule` or `Circle` stays square | — | — |
| Paint | `StateValue<ShapeStyle>` | `scrim` `Role(Scrim)` over the window; `background` `Role(RaisedBackground)` | `title` `Role(Label)`; `message` `Role(SecondaryLabel)` | `symbol` `Role(TertiaryLabel)`; `title` `Role(SecondaryLabel)` |
| `border` | `Option<(ShapeStyle, f32)>` | `Some((Role(Separator), HAIRLINE))`, drawn outside | — | — |
| `effect` | `ColorEffect` | `system_effect` | the same | the same |
| Font | `Font` | — | `title_font` `HEADLINE`, `message_font` `BODY` | `font` `BODY` |

States: only `normal`. Like `Badge`, `Sheet::style` and
`ContentUnavailableView::style` build the view again, so they come right
after `new`.

```rust
Sheet::new(content).style(|s| SheetStyle {
    background: StateValue::new(ShapeStyle::Material(Thickness::Regular)),
    border: None,
    ..s
})
```

### SettingsWindowStyle and AboutViewStyle

`SettingsWindow` is built from styled views: its sidebar takes
`SidebarStyle`, `SidebarItemStyle`, `SidebarSectionStyle` and
`SidebarAccountStyle`, the search field `SearchFieldStyle`, the bar
`TitlebarStyle`, the arrows `ToolGroupStyle` and `IconButtonStyle`, the
split `SplitViewStyle`, the scroll area `ScrollViewStyle` and each pane
`FormStyle`. `SettingsWindowStyle` holds the rest: the gaps between them and
the search popover. `Pane` and `PaneGroup` are data and take no style.

| Field | Type | `SettingsWindowStyle` |
|---|---|---|
| `search_gap`, `toolbar_spacing`, `navigation_spacing` | `f32` | `SETTINGS.sidebar_search_gap` under the search field; `SETTINGS.toolbar_gap`, the `spacing` of the bar; `SPACE_2` between the sidebar toggle and the arrows |
| `popover_gap`, `popover_max_height`, `popover_padding` | `f32` | `SETTINGS.search_popover_gap` under the field, `SETTINGS.search_popover_max_height`, the `padding` of the popover menu palette |
| `header_height`, `result_height`, `result_subtitle_height` | `f32` | `SETTINGS.search_header_height`; `SIDEBAR.item_height`; `SETTINGS.search_result_height` for a result with a pane name under it |
| `result_padding_x`, `result_spacing` | `f32` | `item_padding_x` of the popover menu palette, also for the headers; `SIDEBAR.item_gap` between the tile and the text |
| `popover_shape`, `result_shape` | `Shape` | `RoundedRectangle { radius: LIST_GROUP.radius, style: Circular }`, a `Capsule` or `Circle` stays square; `RoundedRectangle` of the palette's `item_radius` |
| `popover_background` | `StateValue<ShapeStyle>` | `Color` of the popover menu background |
| `header` | `StateValue<ShapeStyle>` | `Role(SecondaryLabel)`, "Suggestions", "Recents" |
| `result_background`, `result_title`, `result_subtitle` | `StateValue<ShapeStyle>` | `Clear`, hovered `Color` of the inactive sidebar selection; `Role(Label)`; `Role(SecondaryLabel)`. While the user types |
| `suggestion_background`, `suggestion_title`, `suggestion_subtitle` | `StateValue<ShapeStyle>` | `Clear`, hovered `Accent`; `Role(Label)`, hovered `Role(OnAccent)`; `Role(SecondaryLabel)`, hovered `Role(OnAccent)`. Before the user types |
| `popover_border` | `Option<(ShapeStyle, f32)>` | `Some((Color of the popover separator, HAIRLINE))`, drawn outside |
| `effect` | `ColorEffect` | `system_effect` |
| `header_font`, `title_font`, `subtitle_font` | `Font` | `CAPTION`, `BODY`, `CAPTION` |

States: a result takes hovered while the pointer or the keyboard cursor is on
it, pressed and focused. Its tile takes `SymbolTileStyle`.

| Field | Type | `AboutViewStyle` |
|---|---|---|
| `padding`, `spacing`, `name_spacing` | `f32` | `DIALOG_PADDING`; `SPACE_3` between the icon, the name block and the copyright; `SPACE_1` between the name and the version |
| `name`, `version`, `copyright` | `StateValue<ShapeStyle>` | `Role(Label)`, `Role(SecondaryLabel)`, `Role(TertiaryLabel)` |
| `effect` | `ColorEffect` | `system_effect` |
| `name_font`, `version_font`, `copyright_font` | `Font` | `TITLE`, `BODY`, `CAPTION` |

States: only `normal`. `NavigationButtons` draws nothing of its own: it is a
`ToolGroup` of two `IconButton`s. `LanguagePicker` is a `PopUpButton`.

### VideoViewStyle

With the `video` feature.

| Field | Type | Tengen |
|---|---|---|
| `shape` | `Shape` | `Rectangle`, the clip of the frame; a `Capsule` or `Circle` rounds by the view's size |

`.corner_radius(r)` is an instance override of `shape` with
`RoundedRectangle { radius: r, style: Circular }`, applied before `.style`.

### MenuStyle

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
[view styles](#textstyle-badgestyle-dividerstyle-avatarstyle-and-imagestyle).

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

Every style implements `ControlStyle`: `tengen()` and the accessibility rules
of its control. The rollout order is in
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
