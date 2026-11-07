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
   - "the sliders in this panel" → container, `.slider_style(..)`;
   - "every slider in the app", "a new theme" → theme, `Theme`.
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
| 2 | Theme | `Application::new("app").theme(Theme::tengen().slider(\|s\| …))` |
| 3 | Container | `VStack::new().child(..).slider_style(\|s\| …)` (outermost first) |
| 4 | Instance | `Slider::stepped("Volume", 10).style(\|s\| …)` |
| 5 | Accessibility | applied by the kit, always |

`.slider_style` wraps the view in a `StyleScope`, so it comes last, after
the children. It reaches every `Slider` rendered inside, also inside nested
components. `Slider::tint(Tint)` is an instance override of `fill`.

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
control; the groups do not.

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
    .slider_style(|s| SliderStyle {
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
        .theme(Theme::tengen().slider(|s| SliderStyle { effect: darker_when_pressed, ..s }))
        .window("Mixer", || Slider::stepped("Volume", 10).value(4))
        .run()
}
```

## Accessibility

The kit applies these after every level; you cannot turn them off:

1. Reduce motion: every `Animation` becomes `None`.
2. Reduce transparency: every `Material` becomes its solid color, without
   blur.
3. Keyboard focus: the focus ring is always drawn; no field turns it off.

Contrast is your job when you use `ShapeStyle::Color`: text on a fixed color
needs 4.5:1 and control parts 3:1 (WCAG 2.2, 1.4.3 and 1.4.11). Roles and
`Accent` already meet it.

## Which controls take a style

| Control | Style | Since |
|---|---|---|
| `Slider` | `SliderStyle` | ADR 0064 pilot |

Every other control still draws only Tengen. Its rollout order is in
[ADR 0064](../decisions/0064-styles-and-themes.md#5-rollout).

## Rules

1. Override with `|s| Style { field: value, ..s }`; never rebuild a whole style.
2. Follow the system with `Role`, `Accent` or `Backdrop`; use `Color` only for
   a color that must stay fixed.
3. New default values go to `ctdesign`, never into `bk`.
4. Do not imitate a control with `rect()` to change its look.
5. Put a style on the smallest level that covers the request.
6. Every name here is in the [glossary](../barrierkit/20-glossary.md).
