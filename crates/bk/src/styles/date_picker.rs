use ctdesign::{
    BODY, CALENDAR_CONTRAST_DARK, CALENDAR_CONTRAST_DARK_TINT, CALENDAR_CONTRAST_LIGHT,
    CALENDAR_DARK, CALENDAR_DARK_TINT, CALENDAR_LIGHT, CAPTION, ControlSize,
    DATE_FIELD_BORDER_CONTRAST_DARK, DATE_FIELD_BORDER_CONTRAST_DARK_TINT,
    DATE_FIELD_BORDER_CONTRAST_LIGHT, DATE_FIELD_BORDER_DARK, DATE_FIELD_BORDER_DARK_TINT,
    DATE_FIELD_BORDER_LIGHT, DATE_FIELD_CONTRAST_DARK, DATE_FIELD_CONTRAST_DARK_TINT,
    DATE_FIELD_CONTRAST_LIGHT, DATE_FIELD_DARK, DATE_FIELD_DARK_TINT, DATE_FIELD_LIGHT,
    DATE_PICKER, Font, HAIRLINE, HEADLINE, MENU_POPOVER, STEPPER,
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
    /// Tengen: `(DATE_PICKER.card_left, card_top, card_right, card_bottom)`: how far the card, its border and
    /// its bottom line sit inside `calendar_size`.
    pub calendar_inset: (f32, f32, f32, f32),
    /// Tengen: `MENU_POPOVER.padding`.
    pub calendar_padding: f32,
    /// Tengen: `DATE_PICKER.header_height`.
    pub header_height: f32,
    /// Tengen: `DATE_PICKER.weekday_height`.
    pub weekday_height: f32,
    /// Tengen: `DATE_PICKER.day_height`.
    pub day_height: f32,
    /// Tengen: `DATE_PICKER.column_width`, of each weekday and each day.
    pub column_width: f32,
    /// Tengen: `(DATE_PICKER.grid_left, DATE_PICKER.grid_gap)`: the space before the columns and
    /// between the line under the weekdays and the first week.
    pub grid_inset: (f32, f32),
    /// Tengen: `(DATE_PICKER.title_inset, DATE_PICKER.title_lift)`: the space before the month and
    /// year, and how far it sits above the center of the header.
    pub title_offset: (f32, f32),
    /// Tengen: `(DATE_PICKER.nav_width, DATE_PICKER.nav_height)`, the triangles of the month
    /// arrows.
    pub nav_size: (f32, f32),
    /// Tengen: `DATE_PICKER.dot_size`, the circle that goes to today.
    pub dot_size: f32,
    /// Tengen: `(DATE_PICKER.nav_pitch, DATE_PICKER.nav_inset)`: the distance between the centers
    /// of the three month controls and the space after the last.
    pub nav_spacing: (f32, f32),
    /// Tengen: `(DATE_PICKER.selected_width, DATE_PICKER.selected_height)`: the highlight of the
    /// chosen day.
    pub selected_size: (f32, f32),
    /// Tengen: `DATE_PICKER.selected_overhang`: how far the highlight goes past the right edge
    /// of its column.
    pub selected_overhang: f32,
    /// Tengen: `Color(CALENDAR.background)` (`CALENDAR_CONTRAST` under Increase Contrast), behind
    /// a graphical date picker.
    pub calendar_background: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color(CALENDAR.border), CALENDAR.border_width))`, drawn inside.
    pub calendar_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `Color(CALENDAR.edge)`, the line along the bottom of the graphical date picker.
    pub calendar_edge: StateValue<ShapeStyle>,
    /// Tengen: `Color(CALENDAR.separator)`, the line under the weekdays.
    pub calendar_separator: StateValue<ShapeStyle>,
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
    /// Tengen: `Role(TertiaryLabel)`.
    pub other_month_day: StateValue<ShapeStyle>,
    /// Tengen: `Role(SelectedContentInactive)`, `Role(SelectedContent)` while the picker has focus.
    pub selected_day: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, `Role(OnSelectedContent)` while the picker has focus.
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
    /// Tengen, with `font`, `title_font`: `BODY`, `HEADLINE`, `CAPTION` at the weight of `BODY` for the
    /// days.
    pub day_font: Font,
    /// Tengen: `CAPTION` at `DATE_PICKER.weekday_font_size` and the weight of `HEADLINE`, for the
    /// weekdays.
    pub weekday_font: Font,
}

impl DatePickerStyle {
    pub(crate) fn outlined(self) -> Self {
        Self {
            height: self.height + 2.0 * HAIRLINE,
            padding_x: self.padding_x + HAIRLINE,
            ..self
        }
    }

    pub(crate) fn sized(self, size: ControlSize) -> Self {
        let row = match size {
            ControlSize::Mini => DATE_PICKER.mini,
            ControlSize::Small => DATE_PICKER.small,
            ControlSize::Regular => return self,
            ControlSize::Large => DATE_PICKER.large,
            ControlSize::ExtraLarge => DATE_PICKER.xl,
        };
        Self {
            height: row.height,
            stepper_gap: row.stepper_gap,
            stepper: StepperStyle {
                arrows_size: (row.stepper_width, row.stepper_height),
                chevron_size: row.stepper_chevron,
                chevron_stroke: row.stepper_chevron_stroke,
                shape: rounded(STEPPER.size(size).radius),
                ..self.stepper
            },
            ..self
        }
    }
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
            calendar_inset: (
                DATE_PICKER.card_left,
                DATE_PICKER.card_top,
                DATE_PICKER.card_right,
                DATE_PICKER.card_bottom,
            ),
            calendar_padding: MENU_POPOVER.padding,
            header_height: DATE_PICKER.header_height,
            weekday_height: DATE_PICKER.weekday_height,
            day_height: DATE_PICKER.day_height,
            column_width: DATE_PICKER.column_width,
            grid_inset: (DATE_PICKER.grid_left, DATE_PICKER.grid_gap),
            title_offset: (DATE_PICKER.title_inset, DATE_PICKER.title_lift),
            nav_size: (DATE_PICKER.nav_width, DATE_PICKER.nav_height),
            dot_size: DATE_PICKER.dot_size,
            nav_spacing: (DATE_PICKER.nav_pitch, DATE_PICKER.nav_inset),
            selected_size: (DATE_PICKER.selected_width, DATE_PICKER.selected_height),
            selected_overhang: DATE_PICKER.selected_overhang,
            calendar_background: StateValue::new(ShapeStyle::Color(per_look(
                [
                    CALENDAR_LIGHT.background,
                    CALENDAR_DARK.background,
                    CALENDAR_DARK_TINT.background,
                ],
                [
                    CALENDAR_CONTRAST_LIGHT.background,
                    CALENDAR_CONTRAST_DARK.background,
                    CALENDAR_CONTRAST_DARK_TINT.background,
                ],
            ))),
            calendar_border: Some((
                ShapeStyle::Color(per_look(
                    [
                        CALENDAR_LIGHT.border,
                        CALENDAR_DARK.border,
                        CALENDAR_DARK_TINT.border,
                    ],
                    [
                        CALENDAR_CONTRAST_LIGHT.border,
                        CALENDAR_CONTRAST_DARK.border,
                        CALENDAR_CONTRAST_DARK_TINT.border,
                    ],
                )),
                per_look(
                    [
                        CALENDAR_LIGHT.border_width,
                        CALENDAR_DARK.border_width,
                        CALENDAR_DARK_TINT.border_width,
                    ],
                    [
                        CALENDAR_CONTRAST_LIGHT.border_width,
                        CALENDAR_CONTRAST_DARK.border_width,
                        CALENDAR_CONTRAST_DARK_TINT.border_width,
                    ],
                ),
            )),
            calendar_edge: StateValue::new(ShapeStyle::Color(per_look(
                [
                    CALENDAR_LIGHT.edge,
                    CALENDAR_DARK.edge,
                    CALENDAR_DARK_TINT.edge,
                ],
                [
                    CALENDAR_CONTRAST_LIGHT.edge,
                    CALENDAR_CONTRAST_DARK.edge,
                    CALENDAR_CONTRAST_DARK_TINT.edge,
                ],
            ))),
            calendar_separator: StateValue::new(ShapeStyle::Color(per_look(
                [
                    CALENDAR_LIGHT.separator,
                    CALENDAR_DARK.separator,
                    CALENDAR_DARK_TINT.separator,
                ],
                [
                    CALENDAR_CONTRAST_LIGHT.separator,
                    CALENDAR_CONTRAST_DARK.separator,
                    CALENDAR_CONTRAST_DARK_TINT.separator,
                ],
            ))),
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
            text: role(Role::Label).disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            separator: role(Role::Label).disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            selected_segment: StateValue::new(ShapeStyle::Accent),
            selected_text: role(Role::OnAccent),
            title: role(Role::Label),
            chevron: role(Role::SecondaryLabel),
            weekday: role(Role::SecondaryLabel),
            day: role(Role::Label),
            other_month_day: role(Role::TertiaryLabel),
            selected_day: role(Role::SelectedContentInactive)
                .focused(ShapeStyle::Role(Role::SelectedContent)),
            selected_day_label: role(Role::Label)
                .focused(ShapeStyle::Role(Role::OnSelectedContent)),
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
            day_font: Font {
                weight: BODY.weight,
                ..CAPTION
            },
            weekday_font: Font {
                size: DATE_PICKER.weekday_font_size,
                weight: HEADLINE.weight,
                ..CAPTION
            },
        }
    }
}
