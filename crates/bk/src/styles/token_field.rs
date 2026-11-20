use ctdesign::{
    BODY, Font, HAIRLINE, TEXT_FIELD, TOKEN_FIELD, TOKEN_FILL_CONTRAST_DARK,
    TOKEN_FILL_CONTRAST_DARK_TINT, TOKEN_FILL_CONTRAST_LIGHT, TOKEN_FILL_DARK,
    TOKEN_FILL_DARK_TINT, TOKEN_FILL_LIGHT,
};

use super::text_field::field_shape;
use super::{
    ColorEffect, ControlStyle, CornerStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    per_look, system_effect,
};

/// States: only `normal`; a token is selected with the arrows or a click.
#[derive(Clone, Debug)]
pub struct TokenFieldStyle {
    /// Tengen: `TEXT_FIELD.height`, the least height and the height of each line.
    pub height: f32,
    /// Tengen: `TOKEN_FIELD.padding_x`, where the first token starts.
    pub padding_x: f32,
    /// Tengen: `TOKEN_FIELD.gap`, between tokens and between wrapped lines.
    pub gap: f32,
    /// Tengen: `TOKEN_FIELD.token_height`.
    pub token_height: f32,
    /// Tengen: `TOKEN_FIELD.token_drop`, how far below the center of its line each token sits.
    pub token_drop: f32,
    /// Tengen: `TOKEN_FIELD.token_padding_x`.
    pub token_padding_x: f32,
    /// Tengen: `TOKEN_FIELD.token_label_lift`, how far the label of a token sits above the center
    /// of the token.
    pub token_label_lift: f32,
    /// Tengen: `TOKEN_FIELD.input_min_width`, the text input after the tokens.
    pub input_width: f32,
    /// Tengen: `RoundedRectangle { radius: TEXT_FIELD.radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `RoundedRectangle { radius: TOKEN_FIELD.token_radius, style: Circular }`.
    pub token_shape: Shape,
    /// Tengen: `Role(TextBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub placeholder: StateValue<ShapeStyle>,
    /// Tengen, with `selected_token`: `Color(TOKEN_FILL)` (`TOKEN_FILL_CONTRAST` under Increase
    /// Contrast), `Accent`.
    pub token: StateValue<ShapeStyle>,
    /// Tengen, with `token`: `Color(TOKEN_FILL)` (`TOKEN_FILL_CONTRAST` under Increase Contrast),
    /// `Accent`.
    pub selected_token: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub token_label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_token_label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(ControlBorder), HAIRLINE))`, drawn outside.
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
        Self {
            height: TEXT_FIELD.height,
            padding_x: TOKEN_FIELD.padding_x,
            gap: TOKEN_FIELD.gap,
            token_height: TOKEN_FIELD.token_height,
            token_drop: TOKEN_FIELD.token_drop,
            token_padding_x: TOKEN_FIELD.token_padding_x,
            token_label_lift: TOKEN_FIELD.token_label_lift,
            input_width: TOKEN_FIELD.input_min_width,
            shape: field_shape(),
            token_shape: Shape::RoundedRectangle {
                radius: TOKEN_FIELD.token_radius,
                style: CornerStyle::Circular,
            },
            background: StateValue::new(ShapeStyle::Role(Role::TextBackground)),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            token: StateValue::new(ShapeStyle::Color(per_look(
                [TOKEN_FILL_LIGHT, TOKEN_FILL_DARK, TOKEN_FILL_DARK_TINT],
                [
                    TOKEN_FILL_CONTRAST_LIGHT,
                    TOKEN_FILL_CONTRAST_DARK,
                    TOKEN_FILL_CONTRAST_DARK_TINT,
                ],
            ))),
            selected_token: StateValue::new(ShapeStyle::Accent),
            token_label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_token_label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            border: Some((ShapeStyle::Role(Role::ControlBorder), HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}
