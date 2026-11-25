use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::animation::use_animated;
use crate::elements::{Overrides, forward_overrides};
use crate::interaction::{control_state, use_interaction};
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    ControlState, DisclosureGroupStyle, GroupBoxStyle, LayoutDirection, Paints, Theme, resolve,
};
use crate::symbol::{Symbol, stroked_symbol};

// A right chevron that turns to point down as `progress` goes from 0 to 1, and moves from `closed` to `open`.
pub(crate) fn turning_chevron(
    chevron: impl IntoElement,
    progress: f32,
    closed: (f32, f32),
    open: (f32, f32),
) -> Rect {
    let at = |from: f32, to: f32| from + (to - from) * progress;
    rect()
        .offset_x(at(closed.0, open.0))
        .offset_y(at(closed.1, open.1))
        .child(rect().rotate(90.0 * progress).child(chevron))
}

// The rows of a disclosure unrolling from under their header: `progress` 1 is the natural height, which `measured` keeps from the last layout.
pub(crate) fn drawer(
    progress: f32,
    mut measured: State<f32>,
    children: impl IntoIterator<Item = Element>,
) -> Rect {
    let height = *measured.read();
    rect()
        .width(Size::fill())
        .height(if progress < 1.0 {
            Size::px(progress * height)
        } else {
            Size::auto()
        })
        .offset_y(-(1.0 - progress) * height)
        .overflow(Overflow::Clip)
        .direction(Direction::Vertical)
        .child(
            rect()
                .width(Size::fill())
                .direction(Direction::Vertical)
                .on_sized(move |event: Event<SizedEventData>| {
                    measured.set_if_modified(event.data().area.size.height);
                })
                .children(children),
        )
}

#[derive(PartialEq)]
pub struct GroupBox {
    label: Option<Cow<'static, str>>,
    children: Vec<Element>,
    style: Theme,
    overrides: Overrides,
}

impl GroupBox {
    pub fn new(label: Option<Cow<'static, str>>) -> Self {
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
        let paints = Paints::current(state);
        let root = rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(style.spacing)
            .a11y_role(AccessibilityRole::Group)
            .maybe(self.label.is_some(), |el| {
                el.a11y_alt(self.label.as_deref().unwrap_or_default())
            })
            .maybe_child(self.label.clone().map(|label| {
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
    label: Cow<'static, str>,
    expanded: bool,
    children: Vec<Element>,
    style: Theme,
    overrides: Overrides,
}

impl DisclosureGroup {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
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
        let paints = Paints::current(state);
        let open = *expanded.read();
        let progress = use_animated(f32::from(u8::from(open)), style.animation);
        let measured = use_state(|| 0.0f32);
        let header = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| node.set_expanded(open))
            .height(Size::px(style.row_height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .on_press(move |_| expanded.toggle())
            .child(
                rect()
                    .width(Size::px(style.lead_width))
                    .center()
                    .child(turning_chevron(
                        stroked_symbol(
                            Symbol::ChevronRight,
                            style.chevron_size.0,
                            style.chevron_size.1,
                            paints.color(style.chevron.resolve(state)),
                        ),
                        progress,
                        style.chevron_offset,
                        style.open_chevron_offset,
                    )),
            )
            .child(text(
                self.label.clone(),
                style.font,
                paints.color(style.label.resolve(state)),
            ));
        let root = rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .child(interaction.focus_ring(header, 0.0))
            .maybe(open || progress > 0.0, |el| {
                el.child(
                    drawer(progress, measured, self.children.clone()).padding(Gaps::new(
                        0.0,
                        0.0,
                        0.0,
                        style.indent,
                    )),
                )
            });
        self.overrides.apply(root)
    }
}
