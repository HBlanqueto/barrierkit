use ctdesign::{BODY, CAPTION, Font, HAIRLINE, SPACE_1, SPACE_2, TEXT_EDITOR};

use crate::styles::{ControlStyle, Role, Shape, ShapeStyle, StateValue};

/// `TextEditor` is drawn like an `NSTextView` in a bordered scroll view: a square
/// field with a 1 line border inside, whose border does not turn to the accent: the focus
/// ring marks the focus. The find bar holds a `SearchField`, two `IconButton`s
/// and a small `Button`, which take their own styles.
///
/// States: hovered, focused.
#[derive(Clone, Debug)]
pub struct TextEditorStyle {
    /// Tengen: `TEXT_EDITOR.padding_x`.
    pub padding_x: f32,
    /// Tengen: `TEXT_EDITOR.padding_y`.
    pub padding_y: f32,
    pub spacing: f32,
    pub find_bar_spacing: f32,
    /// Tengen: `Rectangle`; a `Capsule` rounds by half of one line.
    pub shape: Shape,
    /// Tengen: `Role(WindowBackground)`, which is `textBackgroundColor` of a text view: opaque
    /// white and `1e1e1e`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, the caret too.
    pub text: StateValue<ShapeStyle>,
    /// Tengen: `Role(TertiaryLabel)`.
    pub placeholder: StateValue<ShapeStyle>,
    pub link: StateValue<ShapeStyle>,
    pub misspelling: StateValue<ShapeStyle>,
    pub caption: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(ControlBorder), HAIRLINE))`, drawn inside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `BODY` with the line height `TEXT_EDITOR.line_height`.
    pub font: Font,
    /// Tengen: `CAPTION`.
    pub caption_font: Font,
}

impl ControlStyle for TextEditorStyle {
    fn tengen() -> Self {
        Self {
            padding_x: TEXT_EDITOR.padding_x,
            padding_y: TEXT_EDITOR.padding_y,
            spacing: SPACE_1,
            find_bar_spacing: SPACE_2,
            shape: Shape::Rectangle,
            background: StateValue::new(ShapeStyle::Role(Role::WindowBackground)),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            link: StateValue::new(ShapeStyle::Accent),
            misspelling: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            caption: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            border: Some((ShapeStyle::Role(Role::ControlBorder), HAIRLINE)),
            opacity: 1.0,
            font: Font {
                line_height: TEXT_EDITOR.line_height,
                ..BODY
            },
            caption_font: CAPTION,
        }
    }
}
