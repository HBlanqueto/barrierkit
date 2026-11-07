use ctappearance::Resolved;
use ctdesign::{BODY, Font, HAIRLINE, Rgba, TEXT_FIELD, TOKEN_FIELD};

use super::text_field::field_shape;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct TokenFieldStyle {
    pub height: f32,
    pub padding_x: f32,
    pub gap: f32,
    pub token_height: f32,
    pub token_padding_x: f32,
    pub input_width: f32,
    pub shape: Shape,
    pub token_shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub text: StateValue<ShapeStyle>,
    pub placeholder: StateValue<ShapeStyle>,
    pub token: StateValue<ShapeStyle>,
    pub selected_token: StateValue<ShapeStyle>,
    pub token_label: StateValue<ShapeStyle>,
    pub selected_token_label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for TokenFieldStyle {
    fn tengen() -> Self {
        let accent = crate::style::Palette::current().accent;
        Self {
            height: TEXT_FIELD.height,
            padding_x: TEXT_FIELD.padding_x,
            gap: TOKEN_FIELD.gap,
            token_height: TOKEN_FIELD.token_height,
            token_padding_x: TOKEN_FIELD.token_padding_x,
            input_width: TOKEN_FIELD.input_min_width,
            shape: field_shape(),
            token_shape: Shape::Capsule,
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            token: StateValue::new(ShapeStyle::Color(Rgba {
                a: TOKEN_FIELD.token_opacity,
                ..accent
            })),
            selected_token: StateValue::new(ShapeStyle::Accent),
            token_label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_token_label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}
