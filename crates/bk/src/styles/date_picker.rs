use ctdesign::{
    BODY, CAPTION, DATE_FIELD_BORDER_CONTRAST_DARK, DATE_FIELD_BORDER_CONTRAST_DARK_TINT,
    DATE_FIELD_BORDER_CONTRAST_LIGHT, DATE_FIELD_BORDER_DARK, DATE_FIELD_BORDER_DARK_TINT,
    DATE_FIELD_BORDER_LIGHT, DATE_FIELD_CONTRAST_DARK, DATE_FIELD_CONTRAST_DARK_TINT,
    DATE_FIELD_CONTRAST_LIGHT, DATE_FIELD_DARK, DATE_FIELD_DARK_TINT, DATE_FIELD_LIGHT,
    DATE_PICKER, Font, HAIRLINE, HEADLINE, MENU_POPOVER,
};

use super::button::rounded;
use super::text_field::field_shape;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, StepperStyle,
    per_look, system_effect,
};

/// States: the field is hovered, pressed and focused; the calendar has only
/// `normal`, on the popover background. The arrows draw from its
/// [`StepperStyle`](crate::styles::StepperStyle).
#[derive(Clone, Debug)]
pub struct DatePickerStyle {
    /// Tengen: `DATE_PICKER.height`.
    pub height: f32,
    /// Tengen: `DATE_PICKER.padding_x`.
    pub padding_x: f32,
    /// Tengen: `DATE_PICKER.stepper_gap`, between the field and the arrows.
    pub stepper_gap: f32,
    /// Tengen: `StepperStyle` with the `DATE_PICKER.stepper_*` measures; the arrows are centered on the field.
    pub stepper: StepperStyle,
    /// Tengen: `(DATE_PICKER.calendar_width, DATE_PICKER.calendar_height)`, without the
    /// padding.
    pub calendar_size: (f32, f32),
    /// Tengen: `MENU_POPOVER.padding`.
    pub calendar_padding: f32,
    /// Tengen: `DATE_PICKER.header_height`.
    pub header_height: f32,
    /// Tengen: `DATE_PICKER.weekday_height`.
    pub weekday_height: f32,
    /// Tengen: `CAPTION.size`, the month arrows.
    pub chevron_size: f32,
    /// Tengen, with `segment_shape`, `day_shape`: `RoundedRectangle` of `TEXT_FIELD.radius`,
    /// `DATE_PICKER.segment_radius` and `DATE_PICKER.day_radius`, all `Circular`.
    pub shape: Shape,
    /// Tengen, with `shape`, `day_shape`: `RoundedRectangle` of `TEXT_FIELD.radius`,
    /// `DATE_PICKER.segment_radius` and `DATE_PICKER.day_radius`, all `Circular`.
    pub segment_shape: Shape,
    /// Tengen, with `shape`, `segment_shape`: `RoundedRectangle` of `TEXT_FIELD.radius`,
    /// `DATE_PICKER.segment_radius` and `DATE_PICKER.day_radius`, all `Circular`.
    pub day_shape: Shape,
    /// Tengen, with `text`, `separator`: `Color(DATE_FIELD)` (`DATE_FIELD_CONTRAST` under
    /// Increase Contrast), `Role(Label)`, `Role(Label)` the `/` and `:`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `background`, `separator`: `Color(DATE_FIELD)` (`DATE_FIELD_CONTRAST` under
    /// Increase Contrast), `Role(Label)`, `Role(Label)` the `/` and `:`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen, with `background`, `text`: `Color(DATE_FIELD)` (`DATE_FIELD_CONTRAST` under
    /// Increase Contrast), `Role(Label)`, `Role(Label)` the `/` and `:`.
    pub separator: StateValue<ShapeStyle>,
    /// Tengen, with `selected_text`: `Accent`, `Role(OnAccent)`: the chosen part while the
    /// field has focus.
    pub selected_segment: StateValue<ShapeStyle>,
    /// Tengen, with `selected_segment`: `Accent`, `Role(OnAccent)`: the chosen part while the
    /// field has focus.
    pub selected_text: StateValue<ShapeStyle>,
    /// Tengen, with `chevron`, `weekday`: `Role(Label)` the month, `Role(SecondaryLabel)`,
    /// `Role(SecondaryLabel)`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen, with `title`, `weekday`: `Role(Label)` the month, `Role(SecondaryLabel)`,
    /// `Role(SecondaryLabel)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen, with `title`, `chevron`: `Role(Label)` the month, `Role(SecondaryLabel)`,
    /// `Role(SecondaryLabel)`.
    pub weekday: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub day: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub other_month_day: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub selected_day: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_day_label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color(DATE_FIELD_BORDER), HAIRLINE))`, `DATE_FIELD_BORDER_CONTRAST` under
    /// Increase Contrast.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen, with `effect`: `1.0`, `system_effect`, for the field.
    pub opacity: f32,
    /// Tengen, with `opacity`: `1.0`, `system_effect`, for the field.
    pub effect: ColorEffect,
    /// Tengen, with `title_font`, `day_font`: `BODY`, `HEADLINE`, `CAPTION` for the days and
    /// weekdays.
    pub font: Font,
    /// Tengen, with `font`, `day_font`: `BODY`, `HEADLINE`, `CAPTION` for the days and
    /// weekdays.
    pub title_font: Font,
    /// Tengen, with `font`, `title_font`: `BODY`, `HEADLINE`, `CAPTION` for the days and
    /// weekdays.
    pub day_font: Font,
}

impl ControlStyle for DatePickerStyle {
    fn tengen() -> Self {
        let role = |role| StateValue::new(ShapeStyle::Role(role));
        Self {
            height: DATE_PICKER.height,
            padding_x: DATE_PICKER.padding_x,
            stepper_gap: DATE_PICKER.stepper_gap,
            stepper: StepperStyle {
                arrows_size: (DATE_PICKER.stepper_width, DATE_PICKER.stepper_height),
                chevron_size: DATE_PICKER.stepper_chevron,
                chevron_stroke: DATE_PICKER.stepper_chevron_stroke,
                chevron_offsets: (
                    DATE_PICKER.stepper_chevron_offset_up,
                    DATE_PICKER.stepper_chevron_offset_down,
                ),
                separator_inset: DATE_PICKER.stepper_separator_inset,
                ..StepperStyle::tengen()
            },
            calendar_size: (DATE_PICKER.calendar_width, DATE_PICKER.calendar_height),
            calendar_padding: MENU_POPOVER.padding,
            header_height: DATE_PICKER.header_height,
            weekday_height: DATE_PICKER.weekday_height,
            chevron_size: CAPTION.size as f32,
            shape: field_shape(),
            segment_shape: rounded(DATE_PICKER.segment_radius),
            day_shape: rounded(DATE_PICKER.day_radius),
            background: StateValue::new(ShapeStyle::Color(per_look(
                [DATE_FIELD_LIGHT, DATE_FIELD_DARK, DATE_FIELD_DARK_TINT],
                [
                    DATE_FIELD_CONTRAST_LIGHT,
                    DATE_FIELD_CONTRAST_DARK,
                    DATE_FIELD_CONTRAST_DARK_TINT,
                ],
            ))),
            text: role(Role::Label),
            separator: role(Role::Label),
            selected_segment: StateValue::new(ShapeStyle::Accent),
            selected_text: role(Role::OnAccent),
            title: role(Role::Label),
            chevron: role(Role::SecondaryLabel),
            weekday: role(Role::SecondaryLabel),
            day: role(Role::Label),
            other_month_day: role(Role::SecondaryLabel),
            selected_day: StateValue::new(ShapeStyle::Accent),
            selected_day_label: role(Role::OnAccent),
            border: Some((
                ShapeStyle::Color(per_look(
                    [
                        DATE_FIELD_BORDER_LIGHT,
                        DATE_FIELD_BORDER_DARK,
                        DATE_FIELD_BORDER_DARK_TINT,
                    ],
                    [
                        DATE_FIELD_BORDER_CONTRAST_LIGHT,
                        DATE_FIELD_BORDER_CONTRAST_DARK,
                        DATE_FIELD_BORDER_CONTRAST_DARK_TINT,
                    ],
                )),
                HAIRLINE,
            )),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
            title_font: HEADLINE,
            day_font: CAPTION,
        }
    }
}
