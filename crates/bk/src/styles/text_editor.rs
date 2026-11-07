use ctdesign::{BODY, CAPTION, Font, HAIRLINE, SPACE_1, SPACE_2, TEXT_FIELD};

use super::text_field::field_shape;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

/// `TextEditor` is drawn like a `TextField`
/// with several lines, but its border does not turn to the accent: the focus
/// ring marks the focus. The find bar holds a `SearchField`, two `IconButton`s
/// and a small `Button`, which take their own styles.
///
/// States: hovered, focused.
#[derive(Clone, Debug)]
pub struct TextEditorStyle {
    /// Tengen, with `padding_y`: `TEXT_FIELD.padding_x`; (`TEXT_FIELD.height` −

    /// `BODY.line_height`) / 2.
    pub padding_x: f32,
    /// Tengen, with `padding_x`: `TEXT_FIELD.padding_x`; (`TEXT_FIELD.height` −
    /// `BODY.line_height`) / 2.
    pub padding_y: f32,
    /// Tengen, with `find_bar_spacing`: `SPACE_1` between the find bar and the text, `SPACE_2`
    /// inside the find bar.
    pub spacing: f32,
    /// Tengen, with `spacing`: `SPACE_1` between the find bar and the text, `SPACE_2` inside
    /// the find bar.
    pub find_bar_spacing: f32,
    /// Tengen: `RoundedRectangle { radius: TEXT_FIELD.radius, style: Circular }`; a `Capsule`
    /// rounds by half of one line.
    pub shape: Shape,
    /// Tengen: `Role(ContentBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `placeholder`: `Role(Label)`, the caret too; `Role(TertiaryLabel)`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen, with `text`: `Role(Label)`, the caret too; `Role(TertiaryLabel)`.
    pub placeholder: StateValue<ShapeStyle>,
    /// Tengen, with `misspelling`, `caption`: `Accent`, `Role(Destructive)`,
    /// `Role(SecondaryLabel)` for the match count.
    pub link: StateValue<ShapeStyle>,
    /// Tengen, with `link`, `caption`: `Accent`, `Role(Destructive)`, `Role(SecondaryLabel)`
    /// for the match count.
    pub misspelling: StateValue<ShapeStyle>,
    /// Tengen, with `link`, `misspelling`: `Accent`, `Role(Destructive)`,
    /// `Role(SecondaryLabel)` for the match count.
    pub caption: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`, drawn inside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
    /// Tengen: `CAPTION`.
    pub caption_font: Font,
}

impl ControlStyle for TextEditorStyle {
    fn tengen() -> Self {
        Self {
            padding_x: TEXT_FIELD.padding_x,
            padding_y: (TEXT_FIELD.height - BODY.line_height as f32) / 2.0,
            spacing: SPACE_1,
            find_bar_spacing: SPACE_2,
            shape: field_shape(),
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            link: StateValue::new(ShapeStyle::Accent),
            misspelling: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            caption: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
            caption_font: CAPTION,
        }
    }
}
