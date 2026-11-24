pub use crate::scroll::{
    IdentityModifiers, ScrollController, ScrollGeometry, ScrollPhase, ScrollPosition, ScrollView,
    use_scroll_controller,
};

use ctdesign::{Font, HAIRLINE, Rgba};
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::app::Shortcut;
use crate::backdrop::Tint;
use crate::choices::Choice;
use crate::controlled::commit;
use crate::elements::{Overrides, forward_overrides};
use crate::environment::use_disabled;
use crate::interaction::{control_state, use_interaction};
use crate::layout::ambient;
use crate::level;
use crate::style::{text_style, with_font};
use crate::styles::Painted;
use crate::styles::{
    Axis, ButtonStyle, CheckboxStyle, ControlState, Edge, LayoutDirection, Paints,
    SegmentedControlStyle, ShadowStyle, Shape, ShapeStyle, StateValue, SwitchStyle, TextStyleScope,
    Theme, ToolGroupStyle, resolve,
};
use crate::symbol::{Symbol, stroked_symbol};
use crate::theme::color;

mod button;
mod circular_slider;
mod segmented_control;
mod slider;
mod text_field;
mod toggle;
mod tool_group;

pub use button::{Button, ButtonRole, ControlSize};
pub use circular_slider::CircularSlider;
pub use segmented_control::SegmentedControl;
pub use slider::Slider;
pub use text_field::{SearchField, SecureField, TextField};
pub(crate) use text_field::{default_pre_key, input, use_field_state};
pub use toggle::{Checkbox, Switch};
pub use tool_group::ToolGroup;

pub(crate) fn inner_border(paints: &Paints, (paint, width): &(ShapeStyle, f32)) -> Border {
    Border::new()
        .width(*width)
        .alignment(BorderAlignment::Inner)
        .fill(color(paints.color(paint)))
}

pub(crate) fn outer_hairline(ink: Rgba) -> Border {
    Border::new()
        .width(HAIRLINE)
        .alignment(BorderAlignment::Outer)
        .fill(color(ink))
}

pub(crate) fn shadowed(element: Rect, shadows: &[ShadowStyle]) -> Rect {
    shadows
        .iter()
        .fold(element, |element, shadow| element.shadow(*shadow))
}

pub(crate) fn dimmed(element: Rect, opacity: f32) -> Rect {
    element.maybe(opacity < 1.0, |element| element.opacity(opacity))
}

pub(crate) fn opacity_when(disabled: bool, opacity: f32, disabled_opacity: f32) -> f32 {
    if disabled {
        opacity * disabled_opacity
    } else {
        opacity
    }
}

pub(crate) fn edge_label(
    text: impl Into<Cow<'static, str>>,
    font: Font,
    fill: Fill,
    opacity: f32,
) -> Element {
    let text = with_font(label().text(text), font).max_lines(1).color(fill);
    if opacity < 1.0 {
        rect().opacity(opacity).child(text).into_element()
    } else {
        text.into_element()
    }
}

pub(crate) fn labeled(row: Rect, control: Rect, label: Element, edge: Edge, gap: f32) -> Rect {
    let label_first = match edge {
        Edge::Top => true,
        Edge::Bottom => false,
        Edge::Leading => ambient() == LayoutDirection::LeftToRight,
        Edge::Trailing => ambient() == LayoutDirection::RightToLeft,
    };
    let row = row
        .direction(if matches!(edge, Edge::Top | Edge::Bottom) {
            Direction::Vertical
        } else {
            Direction::Horizontal
        })
        .cross_align(Alignment::Center)
        .spacing(gap);
    if label_first {
        row.child(label).child(control)
    } else {
        row.child(control).child(label)
    }
}
