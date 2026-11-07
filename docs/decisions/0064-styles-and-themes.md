# 0064 — Styles and themes: every visual property, at three levels

Status: **accepted** (2026-11-07). Pilot: `Slider`. Supersedes the "variants,
not free styles" rule of [ADR 0047](0047-barrierkit-in-layers.md) §2 and the
"no runtime override" rule of
[ADR 0011](0011-design-values-as-rust-constants.md): the default values still
live only in `ctdesign`, but an app can now change any of them.

**Goal:** let an app change any visual property of any control (color,
translucency, blur, shape, corner radius, size, shadow, layout order,
orientation, direction, animation, color algorithm), for one control, for a
container or for the whole app, and let a complete theme different from Tengen
be built from those properties.
**Scope:** the design and its rollout. The guide for agents is
[`agents/styles.md`](../../agents/styles.md); the research is in
[`docs/research/styling-apple.md`](../research/styling-apple.md) and
[`docs/research/styling-toolkits.md`](../research/styling-toolkits.md).
**Reader:** anyone who styles an app or adds a control.
**Input:** controls that read `ctdesign` constants directly in `render`.
**Output:** controls that read one resolved style value.

## Context

Controls read their tokens inside `render`. The only per-instance options are
enums (`Tint` on six controls, `ButtonStyle` and `ControlSize` on `Button`), and
only `Backdrop` and `use_appearance` change values for a subtree, and only
colors. Freya 0.4.3 has theme defaults and per-instance partial overrides, but
no inherited container level, string-keyed storage and color-only references
(survey of `freya-components` theming).

Mature toolkits agree on the shape of a solution:

- SwiftUI separates a control from its look: a style turns a configuration
  into a body, lives in the environment and is inherited by every descendant;
  paint (`ShapeStyle`) resolves late against the current environment; the
  accessibility settings are read-only and win.
- Flutter and Compose resolve each field separately with the order instance >
  component theme > default, and keep defaults in one typed object per
  control (`SliderThemeData`, `SliderDefaults`).
- Flutter, Compose, Qt and CSS model state (hovered, pressed, focused,
  disabled) as part of a property's value, and shapes as typed values.
- Compose's `MotionScheme`, CSS transitions and SwiftUI's `Animation` make
  motion a value, never a literal at the call site.
- GTK, CSS and Material keep the user's accessibility choices on top of any
  theme.

## Decision

### 1. One style value per control

Each control has a style struct named after it (`SliderStyle`) that holds
every visual property it draws. Its `tengen()` constructor builds the default
from `ctdesign` constants and the palette in context, so the token stays the
single source of each value.

Every style struct implements the `ControlStyle` trait: `tengen()` and
`accessible(self, &Resolved)`, which applies the user's settings of
section 4 that concern its fields.

Every style struct uses the same groups of fields:

| Group | Properties |
|---|---|
| Size | width, height, part sizes, gaps |
| Shape | the shape of each part (`Shape`) |
| Paint | the paint of each part, per state (`StateValue<ShapeStyle>`) |
| Border and shadow | borders and lists of shadows of each part |
| Effects | opacity, disabled opacity, the color algorithm (`ColorEffect`) |
| Layout | axis, label position (`Edge`), layout direction, rotation |
| Typography | fonts of the text parts |
| Motion | the animation of value and state changes (`Animation`) |

### 2. Three levels, one order

An override is a function from the inherited style to a new style, so it
changes only the fields it names (`|s| SliderStyle { track_height: 10.0, ..s }`).

| Order | Level | How |
|---|---|---|
| 1 | Tengen | `SliderStyle::tengen()` |
| 2 | Theme (whole app) | `Theme::tengen().style(\|s: SliderStyle\| ..)` given to `Application::theme` or `WindowOptions::theme` |
| 3 | Container (subtree) | `.control_style(\|s: SliderStyle\| ..)` on any view |
| 4 | Instance | `Slider::…().style(..)` |
| 5 | Accessibility | `ControlStyle::accessible`, applied last, always (section 4) |

Later levels win. Containers apply from the outermost to the innermost.

`Theme::style` and `.control_style` are generic over `ControlStyle`: the
type of the closure's parameter picks the control. A `Theme` keeps its
overrides in order, each tagged by its style type, so one value holds the
overrides of every control and a new control needs no change to `Theme`.
The instance level is a `Theme` too, held by the control.

Views that are not controls take a `ShapeStyle` through
`.foreground_style`, `.background_style` and `.shadows`, resolved with the
same code as the control paints.

### 3. Value types

| Type | Values | Source of the idea |
|---|---|---|
| `ShapeStyle` | `Role(Role)`, `Accent`, `Color(Rgba)`, `Gradient(Gradient)`, `Backdrop`, `Material(Thickness)`, `Clear` | SwiftUI `ShapeStyle` |
| `Role` | the colors of `Palette` (`Label`, `Separator`, `ControlBackground`, `Destructive`…) | semantic colors |
| `Gradient` | an angle and stops of `ShapeStyle` | SwiftUI `LinearGradient`, `Color.gradient` |
| `Thickness` | `UltraThin`, `Thin`, `Regular`, `Thick`, `UltraThick`: translucency, blur and tint from `ctdesign` | SwiftUI `Material` |
| `StateValue<T>` | a normal value and optional `hovered`, `pressed`, `focused`, `disabled` values | Flutter `WidgetStateProperty` |
| `Shape` | `Rectangle`, `RoundedRectangle { radius, style }`, `UnevenRoundedRectangle`, `Capsule`, `Circle` | SwiftUI shapes |
| `CornerStyle` | `Continuous`, `Circular` | `RoundedCornerStyle` |
| `ShadowStyle` | color, x, y, blur, spread, inner | SwiftUI `ShadowStyle`, CSS `box-shadow` |
| `Animation` | `None`, `Ease { duration_ms }`, `Spring { duration_ms, bounce }`, `Custom { duration_ms, curve }` | SwiftUI `Animation`, `CustomAnimation` |
| `ColorEffect` | a function `(Rgba, ControlState) -> Rgba` | `NSColor.withSystemEffect` |
| `Axis`, `Edge`, `LayoutDirection` | horizontal/vertical; top/bottom/leading/trailing; left-to-right/right-to-left | SwiftUI |

`ShapeStyle::Role`, `Accent` and `Backdrop` follow the system: light, dark,
contrast and the user's accent. `ShapeStyle::Color` does not. That is how an
app chooses, per property, whether a color follows the system.

### 4. Accessibility wins

After every override the kit applies the user's settings:

1. **Reduce motion:** every `Animation` becomes `None`.
2. **Reduce transparency:** every `Material` paint becomes its solid color,
   and so do the kit's own glass surfaces (full-screen bars, `TopGlass`,
   media `Material`, `GlassContainer`).
3. **Focus:** the focus ring is always drawn when the control has keyboard
   focus.

### 5. Rollout

1. Pilot: `Slider`, with every group of section 1, and the shared types,
   `Theme`, `.control_style` and `.style`.
2. Then one control per commit, in the order of
   [5. Controls](../barrierkit/05-controls.md), with its style struct, tokens,
   glossary rows and a recipe line.
3. Then the views and the title bar.

Each step documents only what exists; [`agents/styles.md`](../../agents/styles.md)
lists which controls already take a style.

## Consequences

- An app restyles one control, a container or the whole app with the same
  closure.
- A theme that does not look like Tengen is a `Theme` value: a set of style
  functions, with no copy of Tengen's numbers.
- Each control resolves its style once per render; a container is a
  component, so its style reaches the controls rendered inside it.
- New defaults still go to `ctdesign`; `bk` gains no design literals.
