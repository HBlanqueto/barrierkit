use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::elements::{Overrides, forward_overrides};
use crate::interaction::{control_state, use_interaction};
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    ControlState, DisclosureGroupStyle, GroupBoxStyle, LayoutDirection, Paints, Theme, resolve,
};
use crate::symbol::{Symbol, stroked_symbol};

#[derive(PartialEq)]
pub struct GroupBox {
    label: Option<&'static str>,
    children: Vec<Element>,
    style: Theme,
    overrides: Overrides,
}

impl GroupBox {
    pub fn new(label: Option<&'static str>) -> Self {
        Self {
            label,
            children: Vec::new(),
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(GroupBoxStyle) -> GroupBoxStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }
}

forward_overrides!(GroupBox);

impl Component for GroupBox {
    fn render(&self) -> impl IntoElement {
        let style: GroupBoxStyle = resolve(&self.style);
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let root = rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(style.spacing)
            .a11y_role(AccessibilityRole::Group)
            .maybe(self.label.is_some(), |el| {
                el.a11y_alt(self.label.unwrap_or_default())
            })
            .maybe_child(self.label.map(|label| {
                rect()
                    .height(Size::px(style.label_height))
                    .padding(Gaps::new(0.0, 0.0, 0.0, style.label_inset))
                    .child(text(
                        label,
                        style.font,
                        paints.color(style.label.resolve(state)),
                    ))
            }))
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Vertical)
                    .padding(Gaps::new_all(style.padding))
                    .corner_radius(style.shape.corner_radius(
                        0.0,
                        0.0,
                        LayoutDirection::LeftToRight,
                    ))
                    .painted(&paints, &style.background)
                    .bordered(&paints, style.border.as_ref())
                    .children(self.children.clone()),
            );
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
pub struct DisclosureGroup {
    label: &'static str,
    expanded: bool,
    children: Vec<Element>,
    style: Theme,
    overrides: Overrides,
}

impl DisclosureGroup {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            expanded: false,
            children: Vec::new(),
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(DisclosureGroupStyle) -> DisclosureGroupStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }
}

forward_overrides!(DisclosureGroup);

impl Component for DisclosureGroup {
    fn render(&self) -> impl IntoElement {
        let style: DisclosureGroupStyle = resolve(&self.style);
        let initial = self.expanded;
        let mut expanded = use_state(|| initial);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let open = *expanded.read();
        let chevron = if open {
            Symbol::ChevronDown
        } else {
            Symbol::ChevronRight
        };
        let shift = if open {
            style.open_chevron_offset
        } else {
            style.chevron_offset
        };
        let header = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_expanded(open))
            .height(Size::px(style.row_height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .on_press(move |_| expanded.toggle())
            .child(
                rect().width(Size::px(style.lead_width)).center().child(
                    rect()
                        .offset_x(shift.0)
                        .offset_y(shift.1)
                        .child(stroked_symbol(
                            chevron,
                            style.chevron_size.0,
                            style.chevron_size.1,
                            paints.color(style.chevron.resolve(state)),
                        )),
                ),
            )
            .child(text(
                self.label,
                style.font,
                paints.color(style.label.resolve(state)),
            ));
        let root = rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .child(interaction.focus_ring(header, 0.0))
            .maybe(open, |el| {
                el.child(
                    rect()
                        .width(Size::fill())
                        .direction(Direction::Vertical)
                        .padding(Gaps::new(0.0, 0.0, 0.0, style.indent))
                        .children(self.children.clone()),
                )
            });
        self.overrides.apply(root)
    }
}
