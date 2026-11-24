use ctdesign::{LIST_GROUP, Rgba};
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};

use crate::adornments::Divider;
use std::any::TypeId;

use crate::compact::Stepper;
use crate::controls::{Button, Checkbox, ControlSize, SegmentedControl, Switch, TextField};
use crate::date_picker::DatePicker;
use crate::elements::{Overrides, forward_overrides};
use crate::indicator::CapacityIndicator;
use crate::interaction::{control_state, use_interaction};
use crate::layout::{ambient, resolve_mirrored};
use crate::menu::measure_text_weight;
use crate::patterns::settings::PageLink;
use crate::progress::ProgressView;
use crate::sidebar::SymbolTile;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    CheckboxStyle, ControlState, DatePickerStyle, DividerStyle, FormStyle, LayoutDirection, Paints,
    PopUpButtonStyle, RadioGroupStyle, SegmentedControlStyle, ShapeStyle, SliderStyle, StateValue,
    StepperStyle, StyleScope, SwitchStyle, TextFieldStyle, Theme, resolve,
};
use crate::symbol::{Symbol, symbol};

#[derive(Clone, PartialEq)]
enum Accessory {
    None,
    Control(Element),
    Leading(Element),
    Field(Element),
    Value(Cow<'static, str>),
    Navigation(Callback<(), ()>),
    Page(&'static str),
    Destructive(Callback<(), ()>),
}

#[derive(Clone, PartialEq)]
pub struct Row {
    pub title: Cow<'static, str>,
    pub keywords: &'static [&'static str],
    detail: Option<Cow<'static, str>>,
    symbol: Option<(Symbol, Rgba)>,
    icon: Option<Element>,
    hover_control: Option<Element>,
    accessory: Accessory,
    stacked: bool,
    tight: bool,
}

impl Row {
    pub fn new(title: impl Into<Cow<'static, str>>) -> Self {
        Self {
            title: title.into(),
            keywords: &[],
            detail: None,
            symbol: None,
            icon: None,
            hover_control: None,
            accessory: Accessory::None,
            stacked: false,
            tight: false,
        }
    }

    pub fn value(title: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) -> Self {
        Self::new(title).with(Accessory::Value(value.into()))
    }

    pub fn navigation(
        title: impl Into<Cow<'static, str>>,
        on_press: impl Into<Callback<(), ()>>,
    ) -> Self {
        Self::new(title).with(Accessory::Navigation(on_press.into()))
    }

    pub fn page(title: impl Into<Cow<'static, str>>, pane_id: &'static str) -> Self {
        Self::new(title).with(Accessory::Page(pane_id))
    }

    pub fn destructive(
        title: impl Into<Cow<'static, str>>,
        on_press: impl Into<Callback<(), ()>>,
    ) -> Self {
        Self::new(title).with(Accessory::Destructive(on_press.into()))
    }

    // A bordered button at the start of the row, like a SwiftUI `Button` in a `Form`.
    pub fn button(
        title: impl Into<Cow<'static, str>>,
        on_press: impl Into<Callback<(), ()>>,
    ) -> Self {
        let title = title.into();
        let on_press = on_press.into();
        Self::new(title.clone()).with(Accessory::Leading(
            Button::new(title)
                .on_press(move |_| on_press.call(()))
                .into_element(),
        ))
    }

    pub fn toggle(title: impl Into<Cow<'static, str>>, on: State<bool>) -> Self {
        let title = title.into();
        Self::new(title.clone()).control(Switch::new(title, on).size(ControlSize::Mini))
    }

    // A `TextField` fills the row after its title and ends its text, as in a SwiftUI `Form`. A segmented
    // control, a stepper and a date field draw `control_overhang` above the frame the row measures.
    pub fn control<C: IntoElement + 'static>(self, control: C) -> Self {
        let element = control.into_element();
        let kind = TypeId::of::<C>();
        let overhangs = [
            TypeId::of::<SegmentedControl>(),
            TypeId::of::<Stepper>(),
            TypeId::of::<DatePicker>(),
        ];
        if kind == TypeId::of::<TextField>() {
            return self.with(Accessory::Field(element));
        }
        if kind == TypeId::of::<Checkbox>() {
            return self.with(Accessory::Leading(element));
        }
        let bar = kind == TypeId::of::<ProgressView>();
        let capacity = kind == TypeId::of::<CapacityIndicator>();
        let mut row = self.with(Accessory::Control(if overhangs.contains(&kind) {
            rect()
                .margin(Gaps::new(-LIST_GROUP.control_overhang, 0.0, 0.0, 0.0))
                .child(element)
                .into_element()
        } else if capacity {
            rect()
                .height(Size::px(LIST_GROUP.capacity_frame))
                .main_align(Alignment::End)
                .child(element)
                .into_element()
        } else {
            element
        }));
        row.tight = bar || capacity;
        row
    }

    pub fn detail(mut self, detail: impl Into<Cow<'static, str>>) -> Self {
        self.detail = Some(detail.into());
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
    pub title: Option<Cow<'static, str>>,
    pub rows: Vec<Row>,
    header: Option<Element>,
    illustration: Option<Element>,
    footer: Option<Cow<'static, str>>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            title: None,
            rows: Vec::new(),
            header: None,
            illustration: None,
            footer: None,
        }
    }

    pub fn titled(title: impl Into<Cow<'static, str>>) -> Self {
        Self {
            title: Some(title.into()),
            ..Self::new()
        }
    }

    pub fn header(mut self, view: impl IntoElement) -> Self {
        self.header = Some(view.into_element());
        self
    }

    pub fn illustration(mut self, view: impl IntoElement) -> Self {
        self.illustration = Some(view.into_element());
        self
    }

    pub fn row(mut self, row: Row) -> Self {
        self.rows.push(row);
        self
    }

    pub fn footer(mut self, footer: impl Into<Cow<'static, str>>) -> Self {
        self.footer = Some(footer.into());
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
    pinned: Option<Section>,
    highlighted: Option<Cow<'static, str>>,
    style: Theme,
    overrides: Overrides,
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
            pinned: None,
            highlighted: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
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

    pub fn pinned(mut self, section: Section) -> Self {
        self.pinned = Some(section);
        self
    }

    pub(crate) fn split_pinned(&self) -> (Option<Form>, Form) {
        let scrolling = Form {
            pinned: None,
            ..self.clone()
        };
        let pinned = self.pinned.clone().map(|section| Form {
            sections: vec![section],
            ..scrolling.clone()
        });
        (pinned, scrolling)
    }

    pub fn highlighted(mut self, row_title: Option<Cow<'static, str>>) -> Self {
        self.highlighted = row_title;
        self
    }
}

forward_overrides!(Form);

impl Component for Form {
    fn render(&self) -> impl IntoElement {
        let style: FormStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default());
        let ink =
            |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(ControlState::default()));
        let right_to_left = ambient() == LayoutDirection::RightToLeft;
        let [top, right, bottom, left] = style.header_padding;
        let (right, left) = if right_to_left {
            (left, right)
        } else {
            (right, left)
        };
        if style.columns {
            return self
                .overrides
                .apply(columns(&self.sections, &style, &paints));
        }
        let corner_radius = style
            .shape
            .corner_radius(0.0, 0.0, LayoutDirection::LeftToRight);
        let sections = self.sections.iter().map(|section| {
            let rows = section.rows.iter().enumerate().map(|(index, row)| {
                FormRow {
                    row: row.clone(),
                    first: index == 0,
                    highlighted: self.highlighted == Some(row.title.clone()),
                    style: self.style.clone(),
                }
                .into_element()
            });
            resolve_mirrored(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Vertical)
                    .maybe_child(match (&section.header, section.title.clone()) {
                        (Some(header), _) => Some(
                            rect()
                                .width(Size::fill())
                                .padding(Gaps::new(top, right, bottom, left))
                                .child(header.clone()),
                        ),
                        (None, Some(title)) => Some(
                            rect()
                                .padding(Gaps::new(top, right, bottom, left))
                                .child(text(title, style.header_font, ink(&style.header))),
                        ),
                        (None, None) => None,
                    })
                    .maybe_child(
                        (!section.rows.is_empty() || section.illustration.is_some()).then(|| {
                            rect()
                                .width(Size::fill())
                                .direction(Direction::Vertical)
                                .corner_radius(corner_radius)
                                .overflow(Overflow::Clip)
                                .painted(&paints, &style.background)
                                .bordered(&paints, style.border.as_ref())
                                .maybe_child(section.illustration.clone())
                                .children(rows)
                        }),
                    )
                    .maybe_child(section.footer.clone().map(|footer| {
                        rect()
                            .padding(Gaps::new(bottom, right, 0.0, left))
                            .child(text(footer, style.footer_font, ink(&style.footer)))
                    }))
                    .into_element(),
            )
        });
        let [form_top, form_right, form_bottom, form_left] = style.padding;
        let root = rect()
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .padding(Gaps::new(form_top, form_right, form_bottom, form_left))
            .child(
                rect()
                    .width(Size::fill())
                    .max_width(Size::px(LIST_GROUP.max_width))
                    .direction(Direction::Vertical)
                    .spacing(style.section_spacing)
                    .children(sections),
            );
        self.overrides.apply(root)
    }
}

fn columns(sections: &[Section], style: &FormStyle, paints: &Paints) -> Rect {
    let state = ControlState::default();
    let ink = paints.color(style.title.resolve(state));
    let value_ink = paints.color(style.value.resolve(state));
    let rows: Vec<&Row> = sections.iter().flat_map(|section| &section.rows).collect();
    let weight = FontWeight::from(style.title_font.weight as i32);
    let label_width = rows
        .iter()
        .map(|row| measure_text_weight(&row.title, style.title_font.size as f32, weight))
        .fold(0.0, f32::max)
        .ceil();
    let [top, right, bottom, left] = style.padding;
    rect()
        .width(Size::fill())
        .direction(Direction::Vertical)
        .spacing(style.columns_spacing)
        .padding(Gaps::new(top, right, bottom, left))
        .children(rows.into_iter().map(|row| {
            let control = match &row.accessory {
                Accessory::Control(control)
                | Accessory::Leading(control)
                | Accessory::Field(control) => Some(control.clone()),
                Accessory::Value(value) => {
                    Some(text(value.clone(), style.value_font, value_ink).into_element())
                }
                _ => None,
            };
            resolve_mirrored(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Horizontal)
                    .content(Content::flex())
                    .cross_align(Alignment::Center)
                    .spacing(style.columns_gap)
                    .child(resolve_mirrored(
                        rect()
                            .width(Size::px(label_width))
                            .main_align(Alignment::End)
                            .direction(Direction::Horizontal)
                            .child(text(row.title.clone(), style.title_font, ink))
                            .into_element(),
                    ))
                    .maybe_child(
                        control.map(|control| rect().width(Size::flex(1.0)).child(control)),
                    )
                    .into_element(),
            )
        }))
}

#[derive(PartialEq)]
struct FormRow {
    row: Row,
    first: bool,
    highlighted: bool,
    style: Theme,
}

impl Component for FormRow {
    fn render(&self) -> impl IntoElement {
        let style: FormStyle = resolve(&self.style);
        let right_to_left = ambient() == LayoutDirection::RightToLeft;
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
        let paints = Paints::current(state);
        let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(state));
        let title_ink = match row.accessory {
            Accessory::Destructive(_) => ink(&style.destructive_title),
            _ => ink(&style.title),
        };
        let mut row_width = use_state(|| 0.0f32);
        let slider_width = use_reactive(
            &((*row_width.read() - 2.0 * style.row_padding.0) * LIST_GROUP.slider_share),
        );
        let control_theme = use_memo(move || {
            let slider_width = *slider_width.read();
            Theme::tengen()
                .style(|style: PopUpButtonStyle| style.plain())
                .style(|style: SegmentedControlStyle| SegmentedControlStyle {
                    equal_segments: true,
                    selected: style.selected.vibrant(),
                    ..style
                })
                .style(|style: StepperStyle| StepperStyle {
                    value_position: None,
                    ..style
                })
                .style(|style: DatePickerStyle| style.plain())
                .style(|style: SwitchStyle| SwitchStyle {
                    fill: style.fill.vibrant(),
                    ..style
                })
                .style(|style: CheckboxStyle| CheckboxStyle {
                    fill: style.fill.vibrant(),
                    ..style
                })
                .style(|style: RadioGroupStyle| RadioGroupStyle {
                    fill: style.fill.vibrant(),
                    ..style
                })
                .style(move |style: SliderStyle| SliderStyle {
                    fill: style.fill.vibrant(),
                    width: if slider_width > 0.0 {
                        slider_width
                    } else {
                        style.width
                    },
                    ..style
                })
        });
        let right_to_left_state = use_reactive(&right_to_left);
        let field_theme = use_memo(move || {
            let right_to_left = *right_to_left_state.read();
            Theme::tengen().style(move |style: TextFieldStyle| TextFieldStyle {
                text_align: if right_to_left {
                    TextAlign::Start
                } else {
                    TextAlign::End
                },
                ..style.plain()
            })
        });
        let accessory: Option<Element> = match &row.accessory {
            Accessory::None | Accessory::Destructive(_) => None,
            Accessory::Control(element) | Accessory::Leading(element) => {
                Some(StyleScope::new(control_theme.read().clone(), element.clone()).into_element())
            }
            Accessory::Field(element) => {
                Some(StyleScope::new(field_theme.read().clone(), element.clone()).into_element())
            }
            Accessory::Value(value) => {
                Some(text(value.clone(), style.value_font, ink(&style.value)).into_element())
            }
            Accessory::Navigation(_) | Accessory::Page(_) => Some(
                symbol(
                    if right_to_left {
                        Symbol::ChevronLeft
                    } else {
                        Symbol::ChevronRight
                    },
                    style.chevron_size,
                    ink(&style.chevron),
                )
                .into_element(),
            ),
        };

        let (padding_x, padding_y) = style.row_padding;
        let first_line = style.row_min_height - 2.0 * padding_y;
        let field = matches!(row.accessory, Accessory::Field(_));
        let leading = matches!(row.accessory, Accessory::Leading(_));
        let aligned_to_first_line = matches!(
            row.accessory,
            Accessory::Control(_) | Accessory::Leading(_) | Accessory::Field(_)
        );
        let stacked = row.stacked && aligned_to_first_line;
        let wrapping = |content: Cow<'static, str>, face, ink| {
            resolve_mirrored(
                text(content, face, ink)
                    .width(if field { Size::auto() } else { Size::fill() })
                    .max_lines(None)
                    .into_element(),
            )
        };
        let labels = rect()
            .width(match (stacked, field) {
                (true, _) => Size::fill(),
                (false, true) => Size::auto(),
                (false, false) => Size::flex(1.0),
            })
            .maybe(!field, |el| {
                el.min_width(Size::percent(LIST_GROUP.label_min_share))
            })
            .direction(Direction::Vertical)
            .maybe(aligned_to_first_line, |el| {
                el.padding(Gaps::new(
                    (first_line - style.title_font.line_height as f32) / 2.0,
                    0.0,
                    0.0,
                    0.0,
                ))
            })
            .child(wrapping(row.title.clone(), style.title_font, title_ink))
            .maybe_child(
                row.detail
                    .clone()
                    .map(|detail| wrapping(detail, style.detail_font, ink(&style.detail))),
            );

        let element = rect()
            .on_sized(move |event: Event<SizedEventData>| {
                row_width.set_if_modified(event.area.width())
            })
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
            .spacing(if stacked && row.tight {
                0.0
            } else {
                style.row_spacing
            })
            .painted(&paints, &style.row_background)
            .maybe_child(row.icon.clone().or_else(|| {
                row.symbol
                    .map(|(glyph, tint)| SymbolTile::new(glyph, tint).into_element())
            }))
            .maybe_child((!leading).then(|| resolve_mirrored(labels.into_element())))
            .child(resolve_mirrored(
                rect()
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .maybe(field || leading, |el| el.width(Size::flex(1.0)))
                    .maybe(row.hover_control.is_some(), |accessories| {
                        accessories.spacing(style.row_spacing)
                    })
                    .child(rect().width(Size::px(0.0)).height(Size::px(first_line)))
                    .maybe_child(row.hover_control.clone().filter(|_| *hovered.read()))
                    .maybe_child(accessory)
                    .into_element(),
            ))
            .maybe(row.hover_control.is_some(), |element| {
                element
                    .on_pointer_over(move |_| hovered.set(true))
                    .on_pointer_out(move |_| hovered.set(false))
            })
            .map(pressable, |element, on_press| {
                interaction.inner_focus_ring(
                    interaction
                        .attach(element, true)
                        .a11y_role(AccessibilityRole::Button)
                        .a11y_alt(&*row.title)
                        .on_press(move |_| on_press.call(())),
                )
            })
            .maybe(self.highlighted, |element| {
                element.painted(&paints, &style.highlight)
            });
        let line = style.separator.clone();
        let separator = (!self.first).then(|| {
            rect()
                .width(Size::fill())
                .padding(if right_to_left {
                    Gaps::new(0.0, style.separator_inset, 0.0, padding_x)
                } else {
                    Gaps::new(0.0, padding_x, 0.0, style.separator_inset)
                })
                .child(
                    Divider::new().style(move |divider: DividerStyle| DividerStyle {
                        line: StateValue::new(line.clone()),
                        ..divider
                    }),
                )
        });
        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .maybe_child(separator)
            .child(resolve_mirrored(element.into_element()))
    }
}
