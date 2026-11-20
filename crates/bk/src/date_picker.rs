use std::rc::Rc;

use ctdesign::{HAIRLINE, SYMBOL};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Position, Size};

use crate::compact::stepper_arrows;
use crate::controlled::{commit, use_controlled};
use crate::controls::{dimmed, shadowed};
use crate::date::{CivilDate, CivilDateTime, DatePart, typing_is_done};
use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::platform::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupSpec, Popups,
};
use crate::style::with_font;
use crate::styles::Painted;
use crate::styles::{
    ControlState, DatePickerStyle, LayoutDirection, Paints, ShapeStyle, StateValue, Theme, resolve,
};
use crate::symbol::{Symbol, symbol};

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
    graphical: bool,
    on_change: Option<Callback<CivilDateTime, ()>>,
    style: Theme,
}

impl DatePicker {
    pub fn new(label: &'static str, value: CivilDateTime) -> Self {
        Self {
            label,
            value,
            elements: DateElements::default(),
            graphical: false,
            on_change: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(DatePickerStyle) -> DatePickerStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn elements(mut self, elements: DateElements) -> Self {
        self.elements = elements;
        self
    }

    pub fn graphical(mut self) -> Self {
        self.graphical = true;
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

fn month_short_name(month: u32) -> &'static str {
    match month {
        1 => text!("month-short-1"),
        2 => text!("month-short-2"),
        3 => text!("month-short-3"),
        4 => text!("month-short-4"),
        5 => text!("month-short-5"),
        6 => text!("month-short-6"),
        7 => text!("month-short-7"),
        8 => text!("month-short-8"),
        9 => text!("month-short-9"),
        10 => text!("month-short-10"),
        11 => text!("month-short-11"),
        _ => text!("month-short-12"),
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
        let style: DatePickerStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let value = use_controlled(&self.value);
        let current = *value.read();
        if self.graphical {
            let on_change = self.on_change.clone();
            return rect().child(CalendarView {
                selected: current.date,
                on_pick: Callback::new(move |date: CivilDate| {
                    commit(
                        value,
                        &on_change,
                        CivilDateTime {
                            date,
                            ..*value.peek()
                        },
                    )
                }),
                popup: false,
                style: self.style.clone(),
            });
        }
        let parts = parts(self.elements);
        let last = parts.len() - 1;
        let mut selected = use_state(|| 0usize);
        if *selected.peek() > last {
            selected.set(last);
        }
        let mut typed = use_state(String::new);
        let mut area = use_state(Area::default);
        let on_change = self.on_change.clone();
        let change: Rc<dyn Fn(CivilDateTime)> =
            Rc::new(move |next| commit(value, &on_change, next));
        let picked = change.clone();
        let calendar_style = style.clone();
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
        let line_height = style.font.line_height as f32;
        let segment_radius = style.segment_shape.corner_radius(
            line_height,
            line_height,
            LayoutDirection::LeftToRight,
        );
        let field_radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);

        let mut segments = Vec::new();
        for (index, part) in parts.iter().copied().enumerate() {
            if index > 0 {
                segments.push(
                    with_font(label().text(separator(parts[index - 1], part)), style.font)
                        .max_lines(1)
                        .color(paints.fill(&style.separator))
                        .into_element(),
                );
            }
            let active = highlight && *selected.read() == index;
            let ink = if active {
                &style.selected_text
            } else {
                &style.text
            };
            let pick = pick.clone();
            let calendar_style = calendar_style.clone();
            let is_date = !matches!(part, DatePart::Hour | DatePart::Minute);
            segments.push(
                rect()
                    .corner_radius(segment_radius)
                    .maybe(active, |el| el.painted(&paints, &style.selected_segment))
                    .on_press(move |_| {
                        selected.set(index);
                        typed.set(String::new());
                        if is_date {
                            open_calendar(
                                *area.peek(),
                                value.peek().date,
                                pick.clone(),
                                calendar_style.clone(),
                            );
                        }
                    })
                    .child(
                        with_font(label().text(part_text(current, part)), style.font)
                            .max_lines(1)
                            .color(paints.fill(ink)),
                    )
                    .into_element(),
            );
        }

        let typing_parts = parts.clone();
        let field = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::DateInput)
            .a11y_alt(self.label)
            .height(Size::px(style.height))
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .corner_radius(field_radius)
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
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
            .spacing(style.stepper_gap)
            .child(interaction.focus_ring(
                dimmed(shadowed(field, &style.shadows), style.opacity),
                field_radius.top_left(),
            ))
            .child(stepper_arrows(
                &style.stepper,
                ControlState::default(),
                Some(Rc::new(move || up(1))),
                Some(Rc::new(move || down(-1))),
            ))
    }
}

fn open_calendar(
    field: Area,
    shown: CivilDate,
    on_pick: Callback<CivilDate, ()>,
    style: DatePickerStyle,
) {
    let padding = style.calendar_padding;
    let (width, height) = style.calendar_size;
    let spec = PopupSpec {
        anchor_rect: crate::platform::anchor_rect(field),
        anchor: PopupAnchor::BottomLeft,
        gravity: PopupGravity::BottomRight,
        size: (
            (width + 2.0 * padding).ceil() as u32,
            (height + 2.0 * padding).ceil() as u32,
        ),
        constraint_adjustment: PopupConstraintAdjustment {
            flip_y: true,
            slide_x: true,
            ..PopupConstraintAdjustment::default()
        },
        grab: true,
        ..PopupSpec::default()
    };
    Popups::get().open(
        spec,
        AppComponent::new(Calendar {
            shown,
            on_pick,
            style,
        }),
    );
}

struct Calendar {
    shown: CivilDate,
    on_pick: Callback<CivilDate, ()>,
    style: DatePickerStyle,
}

impl App for Calendar {
    fn render(&self) -> impl IntoElement {
        let menu = crate::styles::popover_style();
        let resolved = self.style.clone();
        rect()
            .expanded()
            .padding(Gaps::new_all(self.style.calendar_padding))
            .background(crate::theme::color(menu.bg))
            .child(CalendarView {
                selected: self.shown,
                on_pick: self.on_pick.clone(),
                popup: true,
                style: Theme::tengen().style(move |_: DatePickerStyle| resolved.clone()),
            })
    }
}

#[derive(PartialEq)]
struct CalendarView {
    selected: CivilDate,
    on_pick: Callback<CivilDate, ()>,
    popup: bool,
    style: Theme,
}

impl Component for CalendarView {
    fn render(&self) -> impl IntoElement {
        let style: DatePickerStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = ControlState {
            focused: self.popup || interaction.focused(),
            ..ControlState::default()
        };
        let paints = Paints::current(state, style.effect);
        let ink = |content: String, paint: &StateValue<ShapeStyle>, font| {
            with_font(label().text(content), font)
                .max_lines(1)
                .color(paints.fill(paint))
        };
        let selected = self.selected;
        let mut month = use_state(|| selected);
        let page = *month.read();
        let (width, height) = style.calendar_size;
        let (grid_left, grid_gap) = style.grid_inset;
        let (selected_width, selected_height) = style.selected_size;
        let day_radius = style.day_shape.corner_radius(
            selected_width,
            selected_height,
            LayoutDirection::LeftToRight,
        );
        let nav_ink = paints.color(style.chevron.resolve(state));
        let (nav_pitch, nav_inset) = style.nav_spacing;
        let nav =
            |kind: Symbol, label: &'static str, change: Rc<dyn Fn(CivilDate) -> CivilDate>| {
                rect()
                    .width(Size::px(nav_pitch))
                    .height(Size::fill())
                    .center()
                    .a11y_role(AccessibilityRole::Button)
                    .a11y_alt(label)
                    .on_press(move |_| {
                        let next = change(*month.peek());
                        month.set(next);
                    })
                    .child(symbol(kind, SYMBOL.grid, nav_ink))
            };
        let header = rect()
            .width(Size::fill())
            .height(Size::px(style.header_height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .content(torin::prelude::Content::flex())
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .padding(Gaps::new(
                        0.0,
                        0.0,
                        2.0 * style.title_offset.1,
                        style.title_offset.0,
                    ))
                    .child(ink(
                        format!("{} {}", month_short_name(page.month), page.year),
                        &style.title,
                        style.title_font,
                    )),
            )
            .child(nav(
                Symbol::TriangleLeft,
                text!("previous-month"),
                Rc::new(|shown| shown.add_months(-1)),
            ))
            .child(nav(
                Symbol::Dot,
                text!("today"),
                Rc::new(|_| CivilDate::today()),
            ))
            .child(nav(
                Symbol::TriangleRight,
                text!("next-month"),
                Rc::new(|shown| shown.add_months(1)),
            ))
            .child(rect().width(Size::px(nav_inset)));
        let column = |content: Element| {
            rect()
                .width(Size::px(style.column_width))
                .direction(Direction::Horizontal)
                .main_align(Alignment::End)
                .child(content)
        };
        let weekdays = rect()
            .height(Size::px(style.weekday_height))
            .padding(Gaps::new(0.0, 0.0, 0.0, grid_left))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .children(weekday_initials().into_iter().map(|initial| {
                column(ink(initial.to_string(), &style.weekday, style.weekday_font).into_element())
                    .into_element()
            }));
        let separator = rect()
            .width(Size::fill())
            .height(Size::px(HAIRLINE))
            .background(paints.fill(&style.calendar_separator));
        let grid = page.month_grid(week_starts_on_monday());
        let rows = grid.chunks(7).map(|week| {
            rect()
                .a11y_role(AccessibilityRole::Row)
                .height(Size::px(style.day_height))
                .padding(Gaps::new(0.0, 0.0, 0.0, grid_left))
                .direction(Direction::Horizontal)
                .children(week.iter().map(|day| {
                    let day = *day;
                    let chosen = day == selected;
                    let day_ink = if chosen {
                        &style.selected_day_label
                    } else if day.month == page.month {
                        &style.day
                    } else {
                        &style.other_month_day
                    };
                    let on_pick = self.on_pick.clone();
                    let popup = self.popup;
                    column(ink(day.day.to_string(), day_ink, style.day_font).into_element())
                        .height(Size::fill())
                        .cross_align(Alignment::Center)
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
                            if popup {
                                PopupHandle::get().close();
                            }
                        })
                        .maybe_child(chosen.then(|| {
                            rect()
                                .position(
                                    Position::new_absolute()
                                        .right(-style.selected_overhang)
                                        .top((style.day_height - selected_height) / 2.0 - 0.5),
                                )
                                .layer(-1)
                                .width(Size::px(selected_width))
                                .height(Size::px(selected_height))
                                .corner_radius(day_radius)
                                .painted(&paints, &style.selected_day)
                        }))
                        .into_element()
                }))
                .into_element()
        });
        let days = rect()
            .a11y_role(AccessibilityRole::Grid)
            .a11y_alt(format!("{} {}", month_name(page.month), page.year))
            .padding(Gaps::new(grid_gap, 0.0, 0.0, 0.0))
            .children(rows);
        let card = interaction
            .attach(rect(), true)
            .width(Size::px(width))
            .height(Size::px(height))
            .direction(Direction::Vertical)
            .maybe(!self.popup, |card| {
                card.painted(&paints, &style.calendar_background)
                    .bordered(&paints, style.calendar_border.as_ref())
            })
            .child(header)
            .child(weekdays)
            .child(separator)
            .child(days);
        if self.popup {
            card
        } else {
            card.child(
                rect()
                    .position(Position::new_absolute().left(0.0).bottom(0.0))
                    .width(Size::fill())
                    .height(Size::px(HAIRLINE))
                    .background(paints.fill(&style.calendar_edge)),
            )
        }
    }
}
