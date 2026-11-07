# Styles and themes: prompt for AI agents

**Goal:** let an AI agent change how a control looks (color, translucency,
blur, shape, corners, size, shadow, layout, direction, animation, color
algorithm) for one control, a container or the whole app, or build a theme
that does not look like Tengen.
**Scope:** the styling system of [ADR 0064](../decisions/0064-styles-and-themes.md).
Writing an app in general is in [`prompt.md`](prompt.md); colors and tokens
are in [4. Appearance and tokens](../barrierkit/04-appearance-and-tokens.md).
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
   - the color, background or shadow of a `Text`, a stack, `rect()`,
     `Badge` or `Divider` → [Paint on views](#paint-on-views).
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
inside nested components. `Slider::tint(Tint)` is an instance override of
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
| `Thickness` | how translucent a material is | `UltraThin`, `Thin`, `Regular`, `Thick`, `UltraThick`; opacity and blur from `MATERIAL` ([4](../barrierkit/04-appearance-and-tokens.md#materials)) |
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
only `ButtonStyle`, `SliderStyle` and `SwitchStyle` have `animation`.

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
| `label_position` | `Option<Edge>` | `None`: the label is only accessible, as in a [form row](../barrierkit/11-views.md); `Leading` and `Trailing` put it beside the switch, `Top` and `Bottom` above and below |
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
The token capsules of a `SearchField` keep the `TOKEN_FIELD` look.

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

Every style implements `ControlStyle`: `tengen()` and the accessibility rules
of its control. Every other control still draws only Tengen. Its rollout order is in
[ADR 0064](../decisions/0064-styles-and-themes.md#5-rollout).

## Rules

1. Override with `|s| Style { field: value, ..s }`; never rebuild a whole style.
   In `Theme::style` and `.control_style`, write the parameter's type:
   `|s: SliderStyle|`.
2. Follow the system with `Role`, `Accent` or `Backdrop`; use `Color` only for
   a color that must stay fixed.
3. New default values go to `ctdesign`, never into `bk`.
4. Do not imitate a control with `rect()` to change its look.
5. Put a style on the smallest level that covers the request.
6. Every name here is in the [glossary](../barrierkit/20-glossary.md).
