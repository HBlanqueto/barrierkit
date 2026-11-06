use std::rc::Rc;

use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Size};
use vow_design::{
    CAPTION, DATE_PICKER, HAIRLINE, HEADLINE, MENU_POPOVER, STEPPER, TEXT, TEXT_FIELD,
};

use crate::compact::stepper_arrows;
use crate::controlled::{commit, use_controlled};
use crate::date::{CivilDate, DatePart};
use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::platform::{
    PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupHandle, PopupSpec, Popups,
};
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct DatePicker {
    label: &'static str,
    date: CivilDate,
    on_change: Option<Callback<CivilDate, ()>>,
}

impl DatePicker {
    pub fn new(label: &'static str, date: CivilDate) -> Self {
        Self {
            label,
            date,
            on_change: None,
        }
    }

    pub fn on_change(mut self, callback: impl Into<Callback<CivilDate, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
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

fn part_text(date: CivilDate, part: DatePart) -> String {
    match part {
        DatePart::Day => format!("{:02}", date.day),
        DatePart::Month => format!("{:02}", date.month),
        DatePart::Year => format!("{:04}", date.year),
    }
}

impl Component for DatePicker {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let date = use_controlled(&self.date);
        let current = *date.read();
        let order = date_order();
        let mut selected = use_state(|| 0usize);
        let mut area = use_state(Area::default);
        let on_change = self.on_change.clone();
        let pick = Callback::new(move |picked: CivilDate| commit(date, &on_change, picked));
        let stepped = pick.clone();
        let step = Rc::new(move |delta: i32| {
            let part = order[*selected.peek()];
            stepped.call(date.peek().shifted(part, delta));
        });
        let (up, down, keys) = (step.clone(), step.clone(), step);
        let highlight = interaction.focused();

        let mut segments = Vec::new();
        for (index, part) in order.into_iter().enumerate() {
            if index > 0 {
                segments.push(text("/", TEXT, palette.secondary_label).into_element());
            }
            let active = highlight && *selected.read() == index;
            let ink = if active {
                palette.on_accent
            } else {
                palette.label
            };
            let pick = pick.clone();
            segments.push(
                rect()
                    .corner_radius(DATE_PICKER.segment_radius)
                    .maybe(active, |el| el.background(color(palette.accent)))
                    .on_press(move |_| {
                        selected.set(index);
                        open_calendar(*area.peek(), *date.peek(), pick.clone());
                    })
                    .child(text(part_text(current, part), TEXT, ink))
                    .into_element(),
            );
        }

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
            .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                Key::Named(NamedKey::ArrowUp) => keys(1),
                Key::Named(NamedKey::ArrowDown) => keys(-1),
                Key::Named(NamedKey::ArrowLeft) => {
                    let index = selected.peek().saturating_sub(1);
                    selected.set(index);
                }
                Key::Named(NamedKey::ArrowRight) => {
                    let index = (*selected.peek() + 1).min(2);
                    selected.set(index);
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
                        palette.tertiary_label
                    };
                    let on_pick = self.on_pick.clone();
                    rect()
                        .width(Size::px(cell_width))
                        .height(Size::fill())
                        .center()
                        .corner_radius(DATE_PICKER.day_radius)
                        .maybe(chosen, |el| el.background(color(palette.accent)))
                        .a11y_role(AccessibilityRole::Button)
                        .on_press(move |_| {
                            on_pick.call(day);
                            PopupHandle::get().close();
                        })
                        .child(text(day.day.to_string(), CAPTION, ink))
                        .into_element()
                }))
                .into_element()
        });
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(Gaps::new_all(MENU_POPOVER.padding))
            .background(color(menu.bg))
            .direction(Direction::Vertical)
            .child(header)
            .child(weekdays)
            .children(rows)
    }
}
