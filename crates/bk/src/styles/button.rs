use ctappearance::Resolved;
use ctdesign::{BODY, BUTTON, Font, HAIRLINE};

use crate::controls::ControlSize;

use super::{
    Animation, ColorEffect, ControlStyle, CornerStyle, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, reduce_motion, system_effect,
};

/// `ButtonStyle::tengen()` is SwiftUI's `.bordered`.
///
/// States: hovered, pressed, focused, disabled.
///
/// The presets are functions of the style, so they go straight into `.style`,
/// `Theme::style` or `.control_style`, and change only their own fields:
///
/// | Preset | SwiftUI | Changes |
/// |---|---|---|
/// | `ButtonStyle::bordered_prominent` | `.borderedProminent` | `background` `Accent` (disabled `Role(ControlBackground)`), `label` `Role(OnAccent)` (disabled `Role(TertiaryLabel)`), `border` `None` |
/// | `ButtonStyle::destructive` | `Button(role: .destructive)` | `background` `Role(DestructiveFill)` (`destructive` at `BUTTON_DESTRUCTIVE_FILL`), `label` `Role(Destructive)`, `border` `None` |
/// | `ButtonStyle::glass` | `.glass` | `background` `Role(RaisedBackground)` (hovered `Role(ControlHover)`, disabled `Role(ControlBackground)`), `border` `(Role(CardBorder), HAIRLINE)`, `shape` `Capsule` |
/// | `ButtonStyle::glass_prominent` | `.glassProminent` | `bordered_prominent`, then the border and shape of `glass` |
/// | `ButtonStyle::borderless` | `.borderless` | `background` `Clear` (hovered `Role(ControlHover)`), `border` `None` |
///
/// `Button::size(ControlSize)` is an instance override of `height`, `padding_x`,
/// `font` and `shape`: `Small` takes the `BUTTON.small_*` tokens, `Regular` the
/// plain ones and `Large` `BUTTON.large_height` with a `Capsule`. A `Capsule`
/// stays a `Capsule` at every size. A button that is `.toggled(true)` draws its
/// style through `bordered_prominent`.
#[derive(Clone, Debug)]
pub struct ButtonStyle {
    /// Tengen: `BUTTON.height`.
    pub height: f32,
    /// Tengen: `BUTTON.padding_x`.
    pub padding_x: f32,
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`, hovered `Role(ControlHover)`, disabled
    /// `Role(ControlBackground)`.
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
    /// Tengen: `None`: the pressed scale jumps; `Ease` or `Spring` animates it.
    pub animation: Animation,
}

pub(super) fn rounded(radius: f32) -> Shape {
    Shape::RoundedRectangle {
        radius,
        style: CornerStyle::Circular,
    }
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
                .disabled(ShapeStyle::Role(Role::ControlBackground)),
            label: StateValue::new(ShapeStyle::Role(Role::OnAccent))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            ..self
        }
    }

    pub fn destructive(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Role(Role::DestructiveFill))
                .disabled(ShapeStyle::Role(Role::ControlBackground)),
            label: StateValue::new(ShapeStyle::Role(Role::Destructive))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            ..self
        }
    }

    pub fn borderless(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlHover))
                .disabled(ShapeStyle::Clear),
            border: None,
            ..self
        }
    }

    pub fn glass(self) -> Self {
        Self {
            background: StateValue::new(ShapeStyle::Role(Role::RaisedBackground))
                .hovered(ShapeStyle::Role(Role::ControlHover))
                .disabled(ShapeStyle::Role(Role::ControlBackground)),
            ..self.glass_rim()
        }
    }

    pub fn glass_prominent(self) -> Self {
        self.bordered_prominent().glass_rim()
    }

    fn glass_rim(self) -> Self {
        Self {
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            shape: Shape::Capsule,
            ..self
        }
    }

    pub(crate) fn sized(self, size: ControlSize) -> Self {
        let (height, padding_x, font, shape) = match size {
            ControlSize::Small => (
                BUTTON.small_height,
                BUTTON.small_padding_x,
                font(BUTTON.small_font_size),
                rounded(BUTTON.small_radius),
            ),
            ControlSize::Regular => (
                BUTTON.height,
                BUTTON.padding_x,
                font(BUTTON.font_size),
                rounded(BUTTON.radius),
            ),
            ControlSize::Large => (
                BUTTON.large_height,
                BUTTON.padding_x,
                font(BUTTON.font_size),
                Shape::Capsule,
            ),
        };
        let shape = if self.shape == Shape::Capsule {
            Shape::Capsule
        } else {
            shape
        };
        Self {
            height,
            padding_x,
            font,
            shape,
            ..self
        }
    }
}

impl ControlStyle for ButtonStyle {
    fn tengen() -> Self {
        Self {
            height: BUTTON.height,
            padding_x: BUTTON.padding_x,
            shape: rounded(BUTTON.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .hovered(ShapeStyle::Role(Role::ControlHover))
                .disabled(ShapeStyle::Role(Role::ControlBackground)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
            font: font(BUTTON.font_size),
            animation: Animation::None,
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
        for size in [ControlSize::Small, ControlSize::Regular, ControlSize::Large] {
            let tengen = ButtonStyle::tengen();
            assert_eq!(tengen.clone().glass().sized(size).shape, Shape::Capsule);
            assert_eq!(tengen.sized(size).glass().shape, Shape::Capsule);
        }
    }
}
