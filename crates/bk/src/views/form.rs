use ctdesign::{LIST_GROUP, Rgba};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};

use crate::adornments::Divider;
use crate::interaction::{control_state, use_interaction};
use crate::patterns::settings::PageLink;
use crate::sidebar::SymbolTile;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    ControlState, FormStyle, LayoutDirection, Paints, ShapeStyle, StateValue, Theme, resolve,
};
use crate::symbol::{Symbol, symbol};

#[derive(Clone, PartialEq)]
enum Accessory {
    None,
    Control(Element),
    Value(String),
    Navigation(Callback<(), ()>),
    Page(&'static str),
    Destructive(Callback<(), ()>),
}

#[derive(Clone, PartialEq)]
pub struct Row {
    pub title: &'static str,
    pub keywords: &'static [&'static str],
    detail: Option<&'static str>,
    symbol: Option<(Symbol, Rgba)>,
    icon: Option<Element>,
    hover_control: Option<Element>,
    accessory: Accessory,
    stacked: bool,
}

impl Row {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            keywords: &[],
            detail: None,
            symbol: None,
            icon: None,
            hover_control: None,
            accessory: Accessory::None,
            stacked: false,
        }
    }

    pub fn value(title: &'static str, value: impl Into<String>) -> Self {
        Self::new(title).with(Accessory::Value(value.into()))
    }

    pub fn navigation(title: &'static str, on_press: impl Into<Callback<(), ()>>) -> Self {
        Self::new(title).with(Accessory::Navigation(on_press.into()))
    }

    pub fn page(title: &'static str, pane_id: &'static str) -> Self {
        Self::new(title).with(Accessory::Page(pane_id))
    }

    pub fn destructive(title: &'static str, on_press: impl Into<Callback<(), ()>>) -> Self {
        Self::new(title).with(Accessory::Destructive(on_press.into()))
    }

    pub fn control(self, control: impl IntoElement) -> Self {
        self.with(Accessory::Control(control.into_element()))
    }

    pub fn detail(mut self, detail: &'static str) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn symbol(mut self, glyph: Symbol, tint: Rgba) -> Self {
        self.symbol = Some((glyph, tint));
        self
    }

    pub fn hover_control(mut self, control: impl IntoElement) -> Self {
        self.hover_control = Some(control.into_element());
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_element());
        self
    }

    pub fn stacked(mut self) -> Self {
        self.stacked = true;
        self
    }

    pub fn keywords(mut self, keywords: &'static [&'static str]) -> Self {
        self.keywords = keywords;
        self
    }

    fn with(mut self, accessory: Accessory) -> Self {
        self.accessory = accessory;
        self
    }
}

#[derive(Clone, PartialEq)]
pub struct Section {
    pub title: Option<&'static str>,
    pub rows: Vec<Row>,
    illustration: Option<Element>,
    footer: Option<&'static str>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            title: None,
            rows: Vec::new(),
            illustration: None,
            footer: None,
        }
    }

    pub fn titled(title: &'static str) -> Self {
        Self {
            title: Some(title),
            ..Self::new()
        }
    }

    pub fn illustration(mut self, view: impl IntoElement) -> Self {
        self.illustration = Some(view.into_element());
        self
    }

    pub fn row(mut self, row: Row) -> Self {
        self.rows.push(row);
        self
    }

    pub fn footer(mut self, footer: &'static str) -> Self {
        self.footer = Some(footer);
        self
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq)]
pub struct Form {
    pub sections: Vec<Section>,
    highlighted: Option<&'static str>,
    style: Theme,
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Form {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            highlighted: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(FormStyle) -> FormStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    pub fn highlighted(mut self, row_title: Option<&'static str>) -> Self {
        self.highlighted = row_title;
        self
    }
}

impl Component for Form {
    fn render(&self) -> impl IntoElement {
        let style: FormStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink =
            |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(ControlState::default()));
        let mut width = use_state(|| f32::MAX);
        let stacked = *width.read() < LIST_GROUP.stack_below;
        let [top, right, bottom, left] = style.header_padding;
        let corner_radius = style
            .shape
            .corner_radius(0.0, 0.0, LayoutDirection::LeftToRight);
        let sections = self.sections.iter().map(|section| {
            let rows = section.rows.iter().enumerate().map(|(index, row)| {
                FormRow {
                    row: row.clone(),
                    first: index == 0,
                    highlighted: self.highlighted == Some(row.title),
                    stacked,
                    style: self.style.clone(),
                }
                .into_element()
            });
            rect()
                .width(Size::fill())
                .direction(Direction::Vertical)
                .maybe_child(section.title.map(|title| {
                    rect()
                        .padding(Gaps::new(top, right, bottom, left))
                        .child(text(title, style.header_font, ink(&style.header)))
                }))
                .child(
                    rect()
                        .width(Size::fill())
                        .direction(Direction::Vertical)
                        .corner_radius(corner_radius)
                        .overflow(Overflow::Clip)
                        .painted(&paints, &style.background)
                        .bordered(&paints, style.border.as_ref())
                        .maybe_child(section.illustration.clone())
                        .children(rows),
                )
                .maybe_child(section.footer.map(|footer| {
                    rect()
                        .padding(Gaps::new(bottom, right, 0.0, left))
                        .child(text(footer, style.footer_font, ink(&style.footer)))
                }))
                .into_element()
        });
        let [form_top, form_right, form_bottom, form_left] = style.padding;
        rect()
            .width(Size::fill())
            .padding(Gaps::new(form_top, form_right, form_bottom, form_left))
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Vertical)
                    .spacing(style.section_spacing)
                    .on_sized(move |event: Event<SizedEventData>| {
                        width.set_if_modified(event.area.size.width);
                    })
                    .children(sections),
            )
    }
}

#[derive(PartialEq)]
struct FormRow {
    row: Row,
    first: bool,
    highlighted: bool,
    stacked: bool,
    style: Theme,
}

impl Component for FormRow {
    fn render(&self) -> impl IntoElement {
        let style: FormStyle = resolve(&self.style);
        let interaction = use_interaction();
        let mut hovered = use_state(|| false);
        let row = &self.row;
        let pressable = match &row.accessory {
            Accessory::Navigation(cb) | Accessory::Destructive(cb) => Some(cb.clone()),
            Accessory::Page(id) => {
                let id = *id;
                PageLink::current().map(|link| Callback::new(move |()| link.open(id)))
            }
            _ => None,
        };
        let state = if pressable.is_some() {
            control_state(interaction, false)
        } else {
            ControlState::default()
        };
        let paints = Paints::current(state, style.effect);
        let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(state));
        let title_ink = match row.accessory {
            Accessory::Destructive(_) => ink(&style.destructive_title),
            _ => ink(&style.title),
        };
        let accessory: Option<Element> = match &row.accessory {
            Accessory::None | Accessory::Destructive(_) => None,
            Accessory::Control(element) => Some(element.clone()),
            Accessory::Value(value) => {
                Some(text(value.clone(), style.value_font, ink(&style.value)).into_element())
            }
            Accessory::Navigation(_) | Accessory::Page(_) => Some(
                symbol(
                    Symbol::ChevronRight,
                    style.chevron_size,
                    ink(&style.chevron),
                )
                .into_element(),
            ),
        };

        let (padding_x, padding_y) = style.row_padding;
        let first_line = style.row_min_height - 2.0 * padding_y;
        let aligned_to_first_line = matches!(row.accessory, Accessory::Control(_));
        let stacked = (self.stacked || row.stacked) && aligned_to_first_line;
        let wrapping = |content: &'static str, face, ink| {
            text(content, face, ink).width(Size::fill()).max_lines(None)
        };
        let mut labels = rect()
            .width(if stacked {
                Size::fill()
            } else {
                Size::flex(1.0)
            })
            .min_width(Size::percent(LIST_GROUP.label_min_share))
            .direction(Direction::Vertical)
            .maybe(aligned_to_first_line, |el| {
                el.padding(Gaps::new(
                    (first_line - style.title_font.line_height as f32) / 2.0,
                    0.0,
                    0.0,
                    0.0,
                ))
            })
            .child(wrapping(row.title, style.title_font, title_ink));
        if let Some(detail) = row.detail {
            labels = labels.child(wrapping(detail, style.detail_font, ink(&style.detail)));
        }

        let mut element = rect()
            .width(Size::fill())
            .direction(if stacked {
                Direction::Vertical
            } else {
                Direction::Horizontal
            })
            .content(Content::flex())
            .cross_align(if aligned_to_first_line {
                Alignment::Start
            } else {
                Alignment::Center
            })
            .padding(Gaps::new(padding_y, padding_x, padding_y, padding_x))
            .spacing(style.row_spacing)
            .painted(&paints, &style.row_background)
            .maybe_child(row.icon.clone().or_else(|| {
                row.symbol
                    .map(|(glyph, tint)| SymbolTile::new(glyph, tint).into_element())
            }))
            .child(labels)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .maybe(row.hover_control.is_some(), |accessories| {
                        accessories.spacing(style.row_spacing)
                    })
                    .child(rect().width(Size::px(0.0)).height(Size::px(first_line)))
                    .maybe_child(row.hover_control.clone().filter(|_| *hovered.read()))
                    .maybe_child(accessory),
            );
        if row.hover_control.is_some() {
            element = element
                .on_pointer_enter(move |_| hovered.set(true))
                .on_pointer_leave(move |_| hovered.set(false));
        }
        if let Some(on_press) = pressable {
            element = interaction
                .attach(element, true)
                .a11y_role(AccessibilityRole::Button)
                .a11y_alt(row.title)
                .on_press(move |_| on_press.call(()));
            element = interaction.inner_focus_ring(element);
        }
        if self.highlighted {
            element = element.painted(&paints, &style.highlight);
        }
        let separator = (!self.first).then(|| {
            rect()
                .width(Size::fill())
                .padding(Gaps::new(0.0, padding_x, 0.0, style.separator_inset))
                .child(Divider::new())
        });
        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .maybe_child(separator)
            .child(element)
    }
}
