use ctdesign::{BODY, DISCLOSURE, GROUP_BOX, HAIRLINE, Rgba, SPACE_1};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::interaction::use_interaction;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct GroupBox {
    label: Option<&'static str>,
    children: Vec<Element>,
}

impl GroupBox {
    pub fn new(label: Option<&'static str>) -> Self {
        Self {
            label,
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }
}

impl Component for GroupBox {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(SPACE_1)
            .a11y_role(AccessibilityRole::Group)
            .maybe(self.label.is_some(), |el| {
                el.a11y_alt(self.label.unwrap_or_default())
            })
            .maybe_child(self.label.map(|label| {
                rect()
                    .padding(Gaps::new(0.0, 0.0, 0.0, GROUP_BOX.label_inset))
                    .child(text(label, BODY, palette.label))
            }))
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Vertical)
                    .padding(Gaps::new_all(GROUP_BOX.padding))
                    .corner_radius(GROUP_BOX.radius)
                    .background(color(Rgba {
                        a: GROUP_BOX.fill_opacity,
                        ..palette.label
                    }))
                    .border(
                        Border::new()
                            .width(HAIRLINE)
                            .alignment(BorderAlignment::Inner)
                            .fill(color(palette.card_border)),
                    )
                    .children(self.children.clone()),
            )
    }
}

#[derive(PartialEq)]
pub struct DisclosureGroup {
    label: &'static str,
    expanded: bool,
    children: Vec<Element>,
}

impl DisclosureGroup {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            expanded: false,
            children: Vec::new(),
        }
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

impl Component for DisclosureGroup {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let initial = self.expanded;
        let mut expanded = use_state(|| initial);
        let interaction = use_interaction();
        let open = *expanded.read();
        let chevron = if open {
            Symbol::ChevronDown
        } else {
            Symbol::ChevronRight
        };
        let header = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_expanded(open))
            .height(Size::px(DISCLOSURE.row_height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .on_press(move |_| expanded.toggle())
            .child(
                rect()
                    .width(Size::px(DISCLOSURE.indent))
                    .center()
                    .child(symbol(chevron, DISCLOSURE.chevron, palette.secondary_label)),
            )
            .child(text(self.label, BODY, palette.label));
        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .child(interaction.focus_ring(header, 0.0))
            .maybe(open, |el| {
                el.child(
                    rect()
                        .width(Size::fill())
                        .direction(Direction::Vertical)
                        .padding(Gaps::new(0.0, 0.0, 0.0, DISCLOSURE.indent))
                        .children(self.children.clone()),
                )
            })
    }
}
