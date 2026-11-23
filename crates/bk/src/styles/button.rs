use ctappearance::Resolved;
use ctdesign::{BODY, BUTTON, BezelRow, DURATION_FAST_MS, Font, GLASS_CONTROL, HAIRLINE};

use crate::controls::ControlSize;
use crate::glass_container::Glass;

use super::{
    Animation, ColorEffect, ControlStyle, CornerStyle, Gradient, GradientColorSpace, Rgba, Role,
    ShadowStyle, Shape, ShapeStyle, StateValue, bezel_palette, glass_ink, pressed_palette,
    reduce_motion, system_effect,
};

/// `ButtonStyle::tengen()` is SwiftUI's `.bordered`.
///
/// States: pressed, focused, disabled; there is no hover look, as in AppKit.
///
/// The presets are functions of the style, so they go straight into `.style`,
/// `Theme::style` or `.control_style`, and change only their own fields:
///
/// | Preset | SwiftUI | Changes |
/// |---|---|---|
/// | `ButtonStyle::bordered_prominent` | `.borderedProminent` | `background` `Accent` (disabled `Role(ControlBackgroundDisabled)`), `label` `Role(OnAccent)` (disabled `Role(TertiaryLabel)`), `border` `None` |
/// | `ButtonStyle::destructive` | `Button(role: .destructive)` | `background` `Role(DestructiveFill)` (`destructive` at `BUTTON_DESTRUCTIVE_FILL`), `label` `Role(Destructive)`, `border` `None` |
/// | `ButtonStyle::glass` | `.glass` | `glass` `Glass::REGULAR`, `label` full black or white on native glass; without native glass `background` `Role(RaisedBackground)` (pressed `Color(PressedPalette.bezel)`, disabled `Role(ControlBackground)`), `border` `(Role(CardBorder), HAIRLINE)`, `shape` `Capsule` |
/// | `ButtonStyle::glass_prominent` | `.glassProminent` | `glass` tinted with the accent, then `bordered_prominent` and the border and shape of `glass` |
/// | `ButtonStyle::borderless` | `.borderless` | `background` `Clear` (pressed `Color(PressedPalette.bezel)`), `border` `None` |
/// | `ButtonStyle::circular` | `.buttonBorderShape(.circle)` | `height` `BUTTON.circular_size`, `padding_x` `0`, `shape` `Circle`, the icon at `BUTTON.circular_icon` |
/// | `ButtonStyle::plain` | `.plain` | `height` one line of `BODY`, `padding_x` `0`, `background` `Clear` |
/// | `ButtonStyle::link` | `.link` | `plain` with `label` `Accent` (pressed `Color(PressedPalette.accent)`) |
/// | `ButtonStyle::small_square` | `NSButton.BezelStyle.smallSquare` | the `BUTTON.bezel.small_square` row, `background` a `BezelPalette` gradient (pressed `square_pressed`), `border` `square_border`, half opaque while disabled |
/// | `ButtonStyle::textured_square` | `texturedSquare` | the `BUTTON.bezel.textured_square` row, `background` a `BezelPalette` gradient (pressed `textured_pressed`), `border` `textured_border` |
/// | `ButtonStyle::textured_rounded` | `texturedRounded`, `toolbar` | the `BUTTON.bezel.textured_rounded` row, `background` `Clear` (pressed `ring_pressed`), `border` `ring` |
/// | `ButtonStyle::inline` | `inline` | the `BUTTON.bezel.inline` row, `background` `inline_fill`, `label` `Role(SecondaryLabel)` in weight 600 |
/// | `ButtonStyle::push_disclosure` | `pushDisclosure`, `roundedDisclosure` | the `BUTTON.bezel.push_disclosure` row, the push fill |
///
/// `Button::size(ControlSize)` is an instance override of `height`, `padding_x`,
/// `font` and `shape`: `BUTTON.size(size)` has the row of each size, and the
/// sizes whose radius is half the height (`Large`, `ExtraLarge`) are a `Capsule`.
/// A `Capsule` stays a `Capsule` at every size, and a `Circle` keeps its size. A button that is
/// `.toggled(true)` draws its style through `toggled`: `bordered_prominent` with the toggle accent.
#[derive(Clone, Debug)]
pub struct ButtonStyle {
    /// Tengen: `BUTTON.height`.
    pub height: f32,
    /// Tengen: `BUTTON.padding_x`.
    pub padding_x: f32,
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Continuous }`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`, pressed `Color(PressedPalette.bezel)`, disabled
    /// `Role(ControlBackgroundDisabled)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, disabled `Role(TertiaryLabel)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`; paint and width, drawn inside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen, with `disabled_opacity`: `1.0`, `1.0`: the disabled look is in the paints.
    pub opacity: f32,
    /// Tengen, with `opacity`: `1.0`, `1.0`: the disabled look is in the paints.
    pub disabled_opacity: f32,
    /// Tengen: `BUTTON.pressed_scale`.
    pub pressed_scale: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY` with `BUTTON.font_size` and `BUTTON.font_weight`; the label is one line
    /// centered in `height`, so `line_height` has no effect.
    pub font: Font,
    /// Tengen: `None`; in a toolbar on native glass `Glass::REGULAR`, with `height`
    /// `GLASS_CONTROL.height` and `shape` `Capsule`. Where the
    /// platform draws the glass (macOS) it replaces `background` and `border`, and in a toolbar a
    /// button with an image shows the image alone.
    pub glass: Option<Glass>,
    /// Tengen: `Ease { duration_ms: DURATION_FAST_MS }`: the pressed scale eases; `None` makes it
    /// jump.
    pub animation: Animation,
}

pub(super) fn rounded(radius: f32) -> Shape {
    Shape::RoundedRectangle {
        radius,
        style: CornerStyle::Circular,
    }
}

pub(super) fn continuous_rounded(radius: f32) -> Shape {
    Shape::RoundedRectangle {
        radius,
        style: CornerStyle::Continuous,
    }
}

pub(super) fn sized_shape(radius: f32, height: f32) -> Shape {
    if 2.0 * radius >= height {
        Shape::Capsule
    } else {
        continuous_rounded(radius)
    }
}

pub(super) fn corner_for(radii: [f32; 5], size: ControlSize) -> f32 {
    radii[size as usize]
}

fn font(size: f32) -> Font {
    Font {
        size: size as u32,
        weight: BUTTON.font_weight as u16,
        ..BODY
    }
}

impl ButtonStyle {
    pub fn bordered_prominent(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Accent)
                .pressed(ShapeStyle::Color(pressed_palette().accent))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            label: StateValue::new(ShapeStyle::Role(Role::OnAccent))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            glass: self.glass.map(Glass::prominent),
            ..self
        }
    }

    pub(crate) fn toggled(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Role(Role::ToggleAccent))
                .pressed(ShapeStyle::Color(pressed_palette().toggle_accent))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            ..self.bordered_prominent()
        }
    }

    pub fn destructive(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Role(Role::DestructiveFill))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            label: StateValue::new(ShapeStyle::Role(Role::Destructive))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            ..self
        }
    }

    pub fn borderless(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Clear)
                .pressed(ShapeStyle::Color(pressed_palette().bezel))
                .disabled(ShapeStyle::Clear),
            border: None,
            ..self
        }
    }

    pub fn circular(self) -> Self {
        Self {
            height: BUTTON.circular_size,
            padding_x: 0.0,
            shape: Shape::Circle,
            font: Font {
                size: BUTTON.circular_icon as u32,
                ..self.font
            },
            ..self
        }
    }

    pub fn glass(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Role(Role::RaisedBackground))
                .pressed(ShapeStyle::Color(pressed_palette().bezel))
                .disabled(ShapeStyle::Role(Role::ControlBackground)),
            label: if crate::glass::native_glass() {
                StateValue::new(ShapeStyle::Color(glass_ink()))
                    .disabled(ShapeStyle::Role(Role::TertiaryLabel))
            } else {
                self.label.clone()
            },
            glass: Some(Glass::REGULAR),
            ..self.glass_rim()
        }
    }

    pub fn plain(self) -> Self {
        Self {
            height: BODY.line_height as f32,
            padding_x: 0.0,
            background: StateValue::new(ShapeStyle::Clear),
            border: None,
            ..self
        }
    }

    pub fn link(self) -> Self {
        Self {
            label: StateValue::new(ShapeStyle::Accent)
                .pressed(ShapeStyle::Color(pressed_palette().accent))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            ..self.plain()
        }
    }

    pub fn small_square(self) -> Self {
        let palette = bezel_palette();
        Self {
            disabled_opacity: BUTTON.bezel.disabled_opacity,
            ..self
                .bezel(BUTTON.bezel.small_square)
                .vertical(palette.square_top, palette.square_bottom)
                .pressed(palette.square_pressed)
                .framed(palette.square_border)
        }
    }

    pub fn textured_square(self) -> Self {
        let palette = bezel_palette();
        self.bezel(BUTTON.bezel.textured_square)
            .vertical(palette.textured_top, palette.textured_bottom)
            .pressed(palette.textured_pressed)
            .framed(palette.textured_border)
    }

    pub fn textured_rounded(self) -> Self {
        let palette = bezel_palette();
        Self {
            background: StateValue::new(ShapeStyle::Clear),
            ..self.bezel(BUTTON.bezel.textured_rounded)
        }
        .pressed(palette.ring_pressed)
        .framed(palette.ring)
    }

    pub fn inline(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Color(bezel_palette().inline_fill)),
            label: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            font: Font {
                weight: 600,
                ..font(BUTTON.bezel.inline.font_size)
            },
            ..self.bezel(BUTTON.bezel.inline)
        }
    }

    pub fn push_disclosure(self) -> Self {
        self.bezel(BUTTON.bezel.push_disclosure)
    }

    fn bezel(self, row: BezelRow) -> Self {
        Self {
            height: row.height,
            padding_x: row.padding_x,
            shape: sized_shape(row.radius, row.height),
            font: font(row.font_size),
            ..self
        }
    }

    fn vertical(self, top: Rgba, bottom: Rgba) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Gradient(
                Gradient::new(
                    0.0,
                    vec![
                        (ShapeStyle::Color(top), 0.0),
                        (ShapeStyle::Color(bottom), 1.0),
                    ],
                )
                .color_space(GradientColorSpace::Device),
            )),
            ..self
        }
    }

    fn pressed(self, color: Rgba) -> Self {
        Self {
            background: self.background.pressed(ShapeStyle::Color(color)),
            ..self
        }
    }

    fn framed(self, color: Rgba) -> Self {
        Self {
            border: Some((ShapeStyle::Color(color), HAIRLINE)),
            ..self
        }
    }

    pub fn glass_prominent(self) -> Self {
        Self {
            glass: Some(Glass::REGULAR),
            ..self
        }
        .bordered_prominent()
        .glass_rim()
    }

    // SwiftUI's native glass button keeps the bordered shape, a capsule only at the large sizes.
    fn glass_rim(self) -> Self {
        Self {
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            shape: if crate::glass::native_glass() {
                self.shape
            } else {
                Shape::Capsule
            },
            ..self
        }
    }

    pub(crate) fn sized(self, size: ControlSize) -> Self {
        if self.shape == Shape::Circle {
            return self;
        }
        let row = BUTTON.size(size);
        let shape = if self.shape == Shape::Capsule {
            Shape::Capsule
        } else {
            sized_shape(row.radius, row.height)
        };
        Self {
            height: row.height,
            padding_x: row.padding_x,
            font: font(row.font_size),
            shape,
            ..self
        }
    }
}

impl ControlStyle for ButtonStyle {
    fn tengen() -> Self {
        let toolbar_glass = crate::compact::toolbar_glass();
        Self {
            height: if toolbar_glass {
                GLASS_CONTROL.height
            } else {
                BUTTON.height
            },
            padding_x: BUTTON.padding_x,
            shape: if toolbar_glass {
                Shape::Capsule
            } else {
                continuous_rounded(BUTTON.radius)
            },
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .pressed(ShapeStyle::Color(pressed_palette().bezel))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
            font: font(BUTTON.font_size),
            animation: Animation::Ease {
                duration_ms: DURATION_FAST_MS,
            },
            glass: toolbar_glass.then_some(Glass::REGULAR),
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glass_button_stays_a_capsule_at_every_size_in_either_order() {
        for size in [
            ControlSize::Mini,
            ControlSize::Small,
            ControlSize::Regular,
            ControlSize::Large,
            ControlSize::ExtraLarge,
        ] {
            let tengen = ButtonStyle::tengen();
            assert_eq!(tengen.clone().glass().sized(size).shape, Shape::Capsule);
            assert_eq!(tengen.sized(size).glass().shape, Shape::Capsule);
        }
    }
}
