use std::rc::Rc;

use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Size};
use vow_design::{
    BODY, CAPTION, DATE_PICKER, HAIRLINE, HEADLINE, MENU_POPOVER, STEPPER, TEXT_FIELD,
};

use crate::compact::stepper_arrows;
use crate::controlled::{commit, use_controlled};
use crate::date::{CivilDate, CivilDateTime, DatePart, typing_is_done};
use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::platform::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupSpec, Popups,
};
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateElements {
    #[default]
    Date,
    Time,
    DateAndTime,
}

#[derive(PartialEq)]
pub struct DatePicker {
    label: &'static str,
    value: CivilDateTime,
    elements: DateElements,
    on_change: Option<Callback<CivilDateTime, ()>>,
}

impl DatePicker {
    pub fn new(label: &'static str, value: CivilDateTime) -> Self {
        Self {
            label,
            value,
            elements: DateElements::default(),
            on_change: None,
        }
    }

    pub fn elements(mut self, elements: DateElements) -> Self {
        self.elements = elements;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<CivilDateTime, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

fn parts(elements: DateElements) -> Vec<DatePart> {
    let time = [DatePart::Hour, DatePart::Minute];
    match elements {
        DateElements::Date => date_order().to_vec(),
        DateElements::Time => time.to_vec(),
        DateElements::DateAndTime => date_order().into_iter().chain(time).collect(),
    }
}

fn date_order() -> [DatePart; 3] {
    match text!("date-order") {
        "mdy" => [DatePart::Month, DatePart::Day, DatePart::Year],
        "ymd" => [DatePart::Year, DatePart::Month, DatePart::Day],
        _ => [DatePart::Day, DatePart::Month, DatePart::Year],
    }
}

fn week_starts_on_monday() -> bool {
    text!("week-start") == "monday"
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => text!("month-1"),
        2 => text!("month-2"),
        3 => text!("month-3"),
        4 => text!("month-4"),
        5 => text!("month-5"),
        6 => text!("month-6"),
        7 => text!("month-7"),
        8 => text!("month-8"),
        9 => text!("month-9"),
        10 => text!("month-10"),
        11 => text!("month-11"),
        _ => text!("month-12"),
    }
}

fn weekday_initials() -> [&'static str; 7] {
    let monday_first = [
        text!("weekday-1"),
        text!("weekday-2"),
        text!("weekday-3"),
        text!("weekday-4"),
        text!("weekday-5"),
        text!("weekday-6"),
        text!("weekday-7"),
    ];
    if week_starts_on_monday() {
        monday_first
    } else {
        std::array::from_fn(|i| monday_first[(i + 6) % 7])
    }
}

fn part_text(value: CivilDateTime, part: DatePart) -> String {
    match part {
        DatePart::Day => format!("{:02}", value.date.day),
        DatePart::Month => format!("{:02}", value.date.month),
        DatePart::Year => format!("{:04}", value.date.year),
        DatePart::Hour => format!("{:02}", value.hour),
        DatePart::Minute => format!("{:02}", value.minute),
    }
}

fn separator(before: DatePart, after: DatePart) -> &'static str {
    match (before, after) {
        (DatePart::Hour, DatePart::Minute) => ":",
        (_, DatePart::Hour) => " ",
        _ => "/",
    }
}

impl Component for DatePicker {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let value = use_controlled(&self.value);
        let current = *value.read();
        let parts = parts(self.elements);
        let last = parts.len() - 1;
        let mut selected = use_state(|| 0usize);
        let mut typed = use_state(String::new);
        let mut area = use_state(Area::default);
        let on_change = self.on_change.clone();
        let change: Rc<dyn Fn(CivilDateTime)> =
            Rc::new(move |next| commit(value, &on_change, next));
        let picked = change.clone();
        let pick = Callback::new(move |date: CivilDate| {
            let next = CivilDateTime {
                date,
                ..*value.peek()
            };
            picked(next)
        });
        let step = {
            let parts = parts.clone();
            let change = change.clone();
            Rc::new(move |delta: i32| {
                let mut typed = typed;
                typed.set(String::new());
                let next = value.peek().shifted(parts[*selected.peek()], delta);
                change(next);
            })
        };
        let (up, down, keys) = (step.clone(), step.clone(), step);
        let highlight = interaction.focused();

        let mut segments = Vec::new();
        for (index, part) in parts.iter().copied().enumerate() {
            if index > 0 {
                segments.push(
                    text(
                        separator(parts[index - 1], part),
                        BODY,
                        palette.secondary_label,
                    )
                    .into_element(),
                );
            }
            let active = highlight && *selected.read() == index;
            let ink = if active {
                palette.on_accent
            } else {
                palette.label
            };
            let pick = pick.clone();
            let is_date = !matches!(part, DatePart::Hour | DatePart::Minute);
            segments.push(
                rect()
                    .corner_radius(DATE_PICKER.segment_radius)
                    .maybe(active, |el| el.background(color(palette.accent)))
                    .on_press(move |_| {
                        selected.set(index);
                        typed.set(String::new());
                        if is_date {
                            open_calendar(*area.peek(), value.peek().date, pick.clone());
                        }
                    })
                    .child(text(part_text(current, part), BODY, ink))
                    .into_element(),
            );
        }

        let typing_parts = parts.clone();
        let field = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::DateInput)
            .a11y_alt(self.label)
            .height(Size::px(TEXT_FIELD.height))
            .padding(Gaps::new(
                0.0,
                TEXT_FIELD.padding_x,
                0.0,
                TEXT_FIELD.padding_x,
            ))
            .corner_radius(TEXT_FIELD.radius)
            .background(color(palette.content_background))
            .border(
                Border::new()
                    .width(HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(color(palette.separator)),
            )
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_key_down(move |event: Event<KeyboardEventData>| match &event.key {
                Key::Named(NamedKey::ArrowUp) => keys(1),
                Key::Named(NamedKey::ArrowDown) => keys(-1),
                Key::Named(NamedKey::ArrowLeft) => {
                    typed.set(String::new());
                    let index = selected.peek().saturating_sub(1);
                    selected.set(index);
                }
                Key::Named(NamedKey::ArrowRight) => {
                    typed.set(String::new());
                    let index = (*selected.peek() + 1).min(last);
                    selected.set(index);
                }
                Key::Character(digit) if digit.chars().all(|c| c.is_ascii_digit()) => {
                    let index = *selected.peek();
                    let part = typing_parts[index];
                    let buffer = format!("{}{digit}", typed.peek());
                    if let Ok(number) = buffer.parse() {
                        let next = value.peek().typed(part, number);
                        change(next);
                    }
                    if typing_is_done(part, &buffer) {
                        typed.set(String::new());
                        selected.set((index + 1).min(last));
                    } else {
                        typed.set(buffer);
                    }
                }
                _ => {}
            })
            .children(segments);
        rect()
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(STEPPER.gap)
            .child(interaction.focus_ring(field, TEXT_FIELD.radius))
            .child(stepper_arrows(
                Some(Rc::new(move || up(1))),
                Some(Rc::new(move || down(-1))),
            ))
    }
}

fn open_calendar(field: Area, shown: CivilDate, on_pick: Callback<CivilDate, ()>) {
    let padding = MENU_POPOVER.padding;
    let spec = PopupSpec {
        anchor_rect: (
            field.origin.x.round() as i32,
            field.origin.y.round() as i32,
            field.size.width.round().max(1.0) as u32,
            field.size.height.round().max(1.0) as u32,
        ),
        anchor: PopupAnchor::BottomLeft,
        gravity: PopupGravity::BottomRight,
        size: (
            (DATE_PICKER.calendar_width + 2.0 * padding).ceil() as u32,
            (DATE_PICKER.calendar_height + 2.0 * padding).ceil() as u32,
        ),
        constraint_adjustment: PopupConstraintAdjustment {
            flip_y: true,
            slide_x: true,
            ..PopupConstraintAdjustment::default()
        },
        grab: true,
        ..PopupSpec::default()
    };
    Popups::get().open(spec, AppComponent::new(Calendar { shown, on_pick }));
}

struct Calendar {
    shown: CivilDate,
    on_pick: Callback<CivilDate, ()>,
}

impl App for Calendar {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let menu = crate::theme::popover_palette(
            &crate::theme::resolved(),
            crate::theme::SurfaceKind::Toplevel,
        );
        let selected = self.shown;
        let mut month = use_state(|| selected);
        let page = *month.read();
        let cell_height =
            (DATE_PICKER.calendar_height - DATE_PICKER.header_height - DATE_PICKER.weekday_height)
                / 6.0;
        let cell_width = DATE_PICKER.calendar_width / 7.0;
        let turn = |symbol_kind: Symbol, label: &'static str, delta: i32| {
            rect()
                .a11y_role(AccessibilityRole::Button)
                .a11y_alt(label)
                .on_press(move |_| {
                    let next = month.peek().add_months(delta);
                    month.set(next);
                })
                .child(symbol(
                    symbol_kind,
                    CAPTION.size as f32,
                    palette.secondary_label,
                ))
        };
        let header = rect()
            .width(Size::fill())
            .height(Size::px(DATE_PICKER.header_height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .content(torin::prelude::Content::flex())
            .child(rect().width(Size::flex(1.0)).child(text(
                format!("{} {}", month_name(page.month), page.year),
                HEADLINE,
                palette.label,
            )))
            .child(turn(Symbol::ChevronLeft, text!("previous-month"), -1))
            .child(turn(Symbol::ChevronRight, text!("next-month"), 1));
        let weekdays = rect()
            .height(Size::px(DATE_PICKER.weekday_height))
            .direction(Direction::Horizontal)
            .children(weekday_initials().into_iter().map(|initial| {
                rect()
                    .width(Size::px(cell_width))
                    .center()
                    .child(text(initial, CAPTION, palette.secondary_label))
                    .into_element()
            }));
        let grid = page.month_grid(week_starts_on_monday());
        let rows = grid.chunks(7).map(|week| {
            rect()
                .a11y_role(AccessibilityRole::Row)
                .height(Size::px(cell_height))
                .direction(Direction::Horizontal)
                .children(week.iter().map(|day| {
                    let day = *day;
                    let chosen = day == selected;
                    let ink = if chosen {
                        palette.on_accent
                    } else if day.month == page.month {
                        palette.label
                    } else {
                        palette.secondary_label
                    };
                    let on_pick = self.on_pick.clone();
                    rect()
                        .width(Size::px(cell_width))
                        .height(Size::fill())
                        .center()
                        .corner_radius(DATE_PICKER.day_radius)
                        .maybe(chosen, |el| el.background(color(palette.accent)))
                        .a11y_role(AccessibilityRole::GridCell)
                        .a11y_alt(text!(
                            "calendar-day",
                            day = day.day.to_string(),
                            month = month_name(day.month),
                            year = day.year.to_string()
                        ))
                        .a11y_builder(move |node| node.set_selected(chosen))
                        .on_press(move |_| {
                            on_pick.call(day);
                            PopupHandle::get().close();
                        })
                        .child(text(day.day.to_string(), CAPTION, ink))
                        .into_element()
                }))
                .into_element()
        });
        let days = rect()
            .a11y_role(AccessibilityRole::Grid)
            .a11y_alt(format!("{} {}", month_name(page.month), page.year))
            .children(rows);
        rect()
            .expanded()
            .padding(Gaps::new_all(MENU_POPOVER.padding))
            .background(color(menu.bg))
            .direction(Direction::Vertical)
            .child(header)
            .child(weekdays)
            .child(days)
    }
}
