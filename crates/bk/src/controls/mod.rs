pub use crate::scroll::{ScrollController, ScrollView, use_scroll_controller};

use freya_components::input::{
    Input, InputColorsThemePartial, InputLayoutThemePartial, InputValidator,
};
use freya_components::theming::macros::Preference;

use ctdesign::{Font, Rgba, TEXT_LINE_HEIGHT};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, CursorPoint, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::backdrop::Tint;
use crate::choices::use_choice;
use crate::context_menu::{Spelling, edit_menu};
use crate::controlled::commit;
use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::level;
use crate::password::PasswordField;
use crate::spelling::{
    InputParagraphs, Squiggles, char_index, dictionary, known, misspelled, utf16_range,
};
use crate::style::with_font;
use crate::styles::Painted;
use crate::styles::{
    Axis, ButtonStyle, CheckboxStyle, ControlState, Edge, LayoutDirection, Paints,
    SearchFieldStyle, SecureFieldStyle, SegmentedControlStyle, ShadowStyle, Shape, ShapeStyle,
    SliderStyle, StateValue, SwitchStyle, TextFieldStyle, Theme, TokenFieldStyle, ToolGroupStyle,
    resolve,
};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;
use zeroize::Zeroizing;

mod button;
mod segmented_control;
mod slider;
mod text_field;
mod toggle;
mod tool_group;

pub use button::{Button, ControlSize};
pub use segmented_control::SegmentedControl;
pub use slider::Slider;
pub use text_field::{SearchField, SecureField, TextField};
pub(crate) use text_field::{default_pre_key, input, text_style};
pub use toggle::{Checkbox, Switch};
pub use tool_group::ToolGroup;

pub(crate) fn inner_border(paints: &Paints, (paint, width): &(ShapeStyle, f32)) -> Border {
    Border::new()
        .width(*width)
        .alignment(BorderAlignment::Inner)
        .fill(color(paints.color(paint)))
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

pub(crate) fn dimmed_label(text: Label, opacity: f32) -> Element {
    if opacity < 1.0 {
        rect().opacity(opacity).child(text).into_element()
    } else {
        text.into_element()
    }
}

pub(crate) fn labeled(
    row: Rect,
    control: Rect,
    label: Element,
    edge: Edge,
    direction: LayoutDirection,
    gap: f32,
) -> Rect {
    let label_first = match edge {
        Edge::Top => true,
        Edge::Bottom => false,
        Edge::Leading => direction == LayoutDirection::LeftToRight,
        Edge::Trailing => direction == LayoutDirection::RightToLeft,
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
