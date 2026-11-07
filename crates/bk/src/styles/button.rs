use ctappearance::Resolved;
use ctdesign::{BODY, BUTTON, Font, HAIRLINE};

use crate::controls::ControlSize;

use super::{
    Animation, ColorEffect, ControlStyle, CornerStyle, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct ButtonStyle {
    pub height: f32,
    pub padding_x: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub pressed_scale: f32,
    pub effect: ColorEffect,
    pub font: Font,
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
