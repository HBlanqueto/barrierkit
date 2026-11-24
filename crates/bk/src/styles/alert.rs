use ctdesign::{
    BODY, DIALOG_ALERT_ICON, DIALOG_ALERT_ICON_GAP, DIALOG_ALERT_LINE_HEIGHT, DIALOG_ALERT_PADDING,
    DIALOG_ALERT_SPACING, DIALOG_ALERT_STACK_SPACING, DIALOG_ALERT_WIDTH, Font, HEADLINE, SPACE_2,
    SPACE_4,
};

use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct AlertStyle {
    /// Tengen: `DIALOG_ALERT_WIDTH` with `padding`.
    pub width: f32,
    /// Tengen: `DIALOG_ALERT_PADDING`, around the buttons and under them; the texts take the
    /// sheet's `padding` at the top and the sides.
    pub padding: f32,
    /// Tengen: `DIALOG_ALERT_SPACING`, between the title, the message and the checkbox.
    pub spacing: f32,
    /// Tengen: `DIALOG_ALERT_ICON`, the square of the icon `Alert::icon` puts above the title.
    pub icon_size: f32,
    /// Tengen: `DIALOG_ALERT_ICON_GAP`, between the icon and the title.
    pub icon_gap: f32,
    /// Tengen: `SPACE_2`.
    pub buttons_spacing: f32,
    /// Tengen: `DIALOG_ALERT_STACK_SPACING`, between the buttons of an alert with three or more.
    pub buttons_stack_spacing: f32,
    /// Tengen: `SPACE_4`, above the buttons, and above the accessory view and the checkbox.
    pub buttons_margin_top: f32,
    /// Tengen: `title` `Role(Label)`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen: `message` `Role(Label)`.
    pub message: StateValue<ShapeStyle>,
    pub title_font: Font,
    pub message_font: Font,
}

impl ControlStyle for AlertStyle {
    fn tengen() -> Self {
        Self {
            width: DIALOG_ALERT_WIDTH,
            padding: DIALOG_ALERT_PADDING,
            spacing: DIALOG_ALERT_SPACING,
            icon_size: DIALOG_ALERT_ICON,
            icon_gap: DIALOG_ALERT_ICON_GAP,
            buttons_spacing: SPACE_2,
            buttons_stack_spacing: DIALOG_ALERT_STACK_SPACING,
            buttons_margin_top: SPACE_4,
            title: StateValue::new(ShapeStyle::Role(Role::Label)),
            message: StateValue::new(ShapeStyle::Role(Role::Label)),
            title_font: Font {
                line_height: DIALOG_ALERT_LINE_HEIGHT,
                ..HEADLINE
            },
            message_font: Font {
                line_height: DIALOG_ALERT_LINE_HEIGHT,
                ..BODY
            },
        }
    }
}
