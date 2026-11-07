use ctdesign::{BODY, Font, HAIRLINE, Rgba, TEXT_FIELD, TOKEN_FIELD};

use super::text_field::field_shape;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

/// States: only `normal`; a token is selected with the arrows or a click.
#[derive(Clone, Debug)]
pub struct TokenFieldStyle {
    /// Tengen, with `padding_x`: `TEXT_FIELD.height`, the least height and the height of each

    /// line, `TEXT_FIELD.padding_x`.
    pub height: f32,
    /// Tengen, with `height`: `TEXT_FIELD.height`, the least height and the height of each
    /// line, `TEXT_FIELD.padding_x`.
    pub padding_x: f32,
    /// Tengen: `TOKEN_FIELD.gap`, between tokens and between wrapped lines.
    pub gap: f32,
    /// Tengen: `TOKEN_FIELD.token_height`.
    pub token_height: f32,
    /// Tengen: `TOKEN_FIELD.token_padding_x`.
    pub token_padding_x: f32,
    /// Tengen: `TOKEN_FIELD.input_min_width`, the text input after the tokens.
    pub input_width: f32,
    /// Tengen: `RoundedRectangle { radius: TEXT_FIELD.radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `Capsule`.
    pub token_shape: Shape,
    /// Tengen: `Role(ContentBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub placeholder: StateValue<ShapeStyle>,
    /// Tengen, with `selected_token`: `Color` of the accent at `TOKEN_FIELD.token_opacity`,
    /// `Accent`.
    pub token: StateValue<ShapeStyle>,
    /// Tengen, with `token`: `Color` of the accent at `TOKEN_FIELD.token_opacity`, `Accent`.
    pub selected_token: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub token_label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_token_label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`, for the text and the tokens.
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
}
