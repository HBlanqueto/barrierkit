use super::*;
use crate::menu::measure_text;
use std::borrow::Cow;

#[derive(PartialEq)]
pub struct SegmentedControl {
    label: Cow<'static, str>,
    segments: Vec<Cow<'static, str>>,
    selected: usize,
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl SegmentedControl {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            segments: Vec::new(),
            selected: 0,
            disabled: false,
            on_change: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn segment(mut self, label: impl Into<Cow<'static, str>>) -> Self {
        self.segments.push(label.into());
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SegmentedControlStyle) -> SegmentedControlStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: SegmentedControlStyle| style.sized(size))
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for SegmentedControl {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

forward_overrides!(SegmentedControl);

impl Component for SegmentedControl {
    fn render(&self) -> impl IntoElement {
        let style: SegmentedControlStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let keyed = use_state(|| false);
        let segments = self.segments.iter().enumerate().map(|(index, text)| {
            Segment {
                text: text.clone(),
                active: choice.is(index),
                disabled: self.disabled,
                divider: index > 0 && !choice.is(index) && !choice.is(index - 1),
                take_focus: choice.is(index) && *keyed.read(),
                on_select: choice.select(index),
                style: self.style.clone(),
            }
            .into_element()
        });
        let count = self.segments.len() as f32;
        let (direction, height) = match style.axis {
            Axis::Horizontal => (Direction::Horizontal, style.segment_height),
            Axis::Vertical => (
                Direction::Vertical,
                count * style.segment_height + (count - 1.0).max(0.0) * style.gap,
            ),
        };
        let height = height + 2.0 * style.padding;

        let group = rect()
            .a11y_role(AccessibilityRole::TabList)
            .a11y_alt(&*self.label)
            .width(Size::fill())
            .direction(direction)
            .content(Content::flex())
            .padding(Gaps::new_all(style.padding))
            .spacing(style.gap)
            .corner_radius(
                style
                    .shape
                    .corner_radius(height, height, LayoutDirection::LeftToRight),
            )
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .children(segments);
        let root = choice.arrow_keys(dimmed(group, style.opacity), self.segments.len(), keyed);
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
struct Segment {
    text: Cow<'static, str>,
    active: bool,
    disabled: bool,
    divider: bool,
    take_focus: bool,
    on_select: Callback<(), ()>,
    style: Theme,
}

impl Component for Segment {
    fn render(&self) -> impl IntoElement {
        let style: SegmentedControlStyle = resolve(&self.style);
        let interaction = use_interaction();
        let paints = Paints::current(control_state(interaction, self.disabled), style.effect);
        crate::choices::take_focus_when(interaction, self.take_focus);
        let active = self.active;
        let on_select = self.on_select.clone();
        let (paint, ink) = if active {
            (&style.selected, &style.selected_label)
        } else {
            (&style.segment, &style.label)
        };
        let font = style.font;
        let height = style.segment_height;
        let radius =
            style
                .segment_shape
                .corner_radius(height, height, LayoutDirection::LeftToRight);

        let segment = interaction
            .attach(rect(), !self.disabled)
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(&*self.text)
            .a11y_builder(move |node| node.set_selected(active))
            .width(match style.axis {
                Axis::Horizontal => Size::flex(
                    measure_text(&self.text, font.size as f32) + 2.0 * style.segment_padding_x,
                ),
                Axis::Vertical => Size::fill(),
            })
            .height(Size::px(height))
            .corner_radius(radius)
            .center()
            .on_press(move |_| on_select.call(()))
            .child(
                rect()
                    .padding(Gaps::new(
                        0.0,
                        style.segment_padding_x,
                        0.0,
                        style.segment_padding_x,
                    ))
                    .child(
                        label()
                            .text(self.text.clone())
                            .max_lines(1)
                            .font_family(font.family)
                            .font_size(font.size as f32)
                            .font_weight(font.weight as i32)
                            .color(paints.fill(ink)),
                    ),
            )
            .painted(&paints, paint)
            .maybe(self.divider, |el| {
                el.child(
                    rect()
                        .position(
                            Position::new_absolute()
                                .left(0.0)
                                .top((height - style.divider_height) / 2.0),
                        )
                        .width(Size::px(HAIRLINE))
                        .height(Size::px(style.divider_height))
                        .background(paints.fill(&style.divider)),
                )
            });
        let segment = if active {
            shadowed(segment, &style.selected_shadows)
        } else {
            segment
        };
        interaction.focus_ring(segment, radius.top_left())
    }
}
