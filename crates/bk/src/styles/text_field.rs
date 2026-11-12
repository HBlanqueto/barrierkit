use ctdesign::{BODY, Font, TEXT_FIELD};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

/// Freya's `Input` draws `TextField` and `SearchField`. It paints one color per
/// part: a `Gradient` gives its first stop, `Backdrop` the first backdrop color
/// and a `Material` its tint, with the blur drawn by the kit. It draws the
/// border 1 wide, and 2 wide while focused.
///
/// States: `TextField` and `SearchField` are hovered and focused;
/// `SecureField` is hovered, pressed and focused and draws the kit's focus ring.
/// The token capsules of a `SearchField` draw from the
/// [`TokenFieldStyle`](crate::styles::TokenFieldStyle) in context, and the field of a
/// `ComboBox` from the `TextFieldStyle` in context.
#[derive(Clone, Debug)]
pub struct TextFieldStyle {
    /// Tengen: `TEXT_FIELD.height`.
    pub height: f32,
    /// Tengen: `TEXT_FIELD.padding_x`.
    pub padding_x: f32,
    /// Tengen: `RoundedRectangle { radius: TEXT_FIELD.radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `Role(ContentBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Separator)`, focused `Accent`.
    pub border: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub placeholder: StateValue<ShapeStyle>,
    /// Tengen: `Role(Destructive)`, the spelling squiggle.
    pub misspelling: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`; the text sits (`height` − `line_height`) / 2 from the top.
    pub font: Font,
}

pub(super) fn field_shape() -> Shape {
    rounded(TEXT_FIELD.radius)
}

impl ControlStyle for TextFieldStyle {
    fn tengen() -> Self {
        Self {
            height: TEXT_FIELD.height,
            padding_x: TEXT_FIELD.padding_x,
            shape: field_shape(),
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            border: StateValue::new(ShapeStyle::Role(Role::Separator)).focused(ShapeStyle::Accent),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            misspelling: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}
