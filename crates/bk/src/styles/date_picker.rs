use ctdesign::{
    BODY, CAPTION, DATE_PICKER, Font, HAIRLINE, HEADLINE, MENU_POPOVER, STEPPER, TEXT_FIELD,
};

use super::button::rounded;
use super::text_field::field_shape;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct DatePickerStyle {
    pub height: f32,
    pub padding_x: f32,
    pub stepper_gap: f32,
    pub calendar_size: (f32, f32),
    pub calendar_padding: f32,
    pub header_height: f32,
    pub weekday_height: f32,
    pub chevron_size: f32,
    pub shape: Shape,
    pub segment_shape: Shape,
    pub day_shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub text: StateValue<ShapeStyle>,
    pub separator: StateValue<ShapeStyle>,
    pub selected_segment: StateValue<ShapeStyle>,
    pub selected_text: StateValue<ShapeStyle>,
    pub title: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub weekday: StateValue<ShapeStyle>,
    pub day: StateValue<ShapeStyle>,
    pub other_month_day: StateValue<ShapeStyle>,
    pub selected_day: StateValue<ShapeStyle>,
    pub selected_day_label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
    pub title_font: Font,
    pub day_font: Font,
}

impl ControlStyle for DatePickerStyle {
    fn tengen() -> Self {
        let role = |role| StateValue::new(ShapeStyle::Role(role));
        Self {
            height: TEXT_FIELD.height,
            padding_x: TEXT_FIELD.padding_x,
            stepper_gap: STEPPER.gap,
            calendar_size: (DATE_PICKER.calendar_width, DATE_PICKER.calendar_height),
            calendar_padding: MENU_POPOVER.padding,
            header_height: DATE_PICKER.header_height,
            weekday_height: DATE_PICKER.weekday_height,
            chevron_size: CAPTION.size as f32,
            shape: field_shape(),
            segment_shape: rounded(DATE_PICKER.segment_radius),
            day_shape: rounded(DATE_PICKER.day_radius),
            background: role(Role::ContentBackground),
            text: role(Role::Label),
            separator: role(Role::SecondaryLabel),
            selected_segment: StateValue::new(ShapeStyle::Accent),
            selected_text: role(Role::OnAccent),
            title: role(Role::Label),
            chevron: role(Role::SecondaryLabel),
            weekday: role(Role::SecondaryLabel),
            day: role(Role::Label),
            other_month_day: role(Role::SecondaryLabel),
            selected_day: StateValue::new(ShapeStyle::Accent),
            selected_day_label: role(Role::OnAccent),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
            title_font: HEADLINE,
            day_font: CAPTION,
        }
    }
}
