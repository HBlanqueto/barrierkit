use std::time::Duration;

use ctappearance::backdrop::Backdrop as Tinted;
pub use ctdesign::Rgba;
use ctdesign::{MATERIAL, MaterialThickness};
use freya_core::prelude::*;

use crate::backdrop::Tint;
use crate::style::Palette;
use crate::theme::color;

mod button;
mod checkbox;
mod choice_cards;
mod color_well;
mod combo_button;
mod control_group;
mod help_link;
mod icon_button;
mod level_slider;
mod menu_button;
mod radio_group;
mod search_field;
mod secure_field;
mod segmented_control;
mod slider;
mod stepper;
mod switch;
mod text_field;
mod theme;
mod toggle_group;
mod tool_group;
mod toolbar_search;

pub use button::ButtonStyle;
pub use checkbox::CheckboxStyle;
pub use choice_cards::ChoiceCardsStyle;
pub use color_well::ColorWellStyle;
pub use combo_button::ComboButtonStyle;
pub use control_group::ControlGroupStyle;
pub use help_link::HelpLinkStyle;
pub use icon_button::IconButtonStyle;
pub use level_slider::LevelSliderStyle;
pub use menu_button::MenuButtonStyle;
pub use radio_group::RadioGroupStyle;
pub use search_field::SearchFieldStyle;
pub use secure_field::SecureFieldStyle;
pub use segmented_control::SegmentedControlStyle;
pub use slider::SliderStyle;
pub use stepper::StepperStyle;
pub use switch::SwitchStyle;
pub use text_field::TextFieldStyle;
pub use theme::{ControlStyle, StyleScope, Theme};
pub(crate) use theme::{resolve, use_provide_theme};
pub use toggle_group::ToggleGroupStyle;
pub use tool_group::ToolGroupStyle;
pub use toolbar_search::ToolbarSearchStyle;

#[derive(Clone, Debug, PartialEq)]
pub enum ShapeStyle {
    Role(Role),
    Accent,
    Color(Rgba),
    Gradient(Gradient),
    Backdrop,
    Material(Thickness),
    Clear,
}

impl From<Tint> for ShapeStyle {
    fn from(tint: Tint) -> Self {
        match tint {
            Tint::Accent => Self::Accent,
            Tint::Color(rgba) => Self::Color(rgba),
            Tint::Backdrop => Self::Backdrop,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Label,
    SecondaryLabel,
    TertiaryLabel,
    Separator,
    ControlBorder,
    WindowBackground,
    ContentBackground,
    ControlBackground,
    ControlHover,
    AccentInactive,
    OnAccent,
    Destructive,
    RaisedBackground,
    SunkenBackground,
    Scrim,
    SelectedControl,
    CardBorder,
}

impl Role {
    fn color(self, palette: &Palette) -> Rgba {
        match self {
            Self::Label => palette.label,
            Self::SecondaryLabel => palette.secondary_label,
            Self::TertiaryLabel => palette.tertiary_label,
            Self::Separator => palette.separator,
            Self::ControlBorder => palette.control_border,
            Self::WindowBackground => palette.window_background,
            Self::ContentBackground => palette.content_background,
            Self::ControlBackground => palette.control_background,
            Self::ControlHover => palette.control_hover,
            Self::AccentInactive => palette.accent_inactive,
            Self::OnAccent => palette.on_accent,
            Self::Destructive => palette.destructive,
            Self::RaisedBackground => palette.raised_background,
            Self::SunkenBackground => palette.sunken_background,
            Self::Scrim => palette.scrim,
            Self::SelectedControl => palette.selected_control,
            Self::CardBorder => palette.card_border,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    angle: f32,
    stops: Vec<(ShapeStyle, f32)>,
}

impl Gradient {
    pub fn new(angle: f32, stops: Vec<(ShapeStyle, f32)>) -> Self {
        Self { angle, stops }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Thickness {
    UltraThin,
    Thin,
    Regular,
    Thick,
    UltraThick,
}

impl Thickness {
    fn metrics(self) -> MaterialThickness {
        match self {
            Self::UltraThin => MATERIAL.ultra_thin,
            Self::Thin => MATERIAL.thin,
            Self::Regular => MATERIAL.regular,
            Self::Thick => MATERIAL.thick,
            Self::UltraThick => MATERIAL.ultra_thick,
        }
    }

    fn material(self, glass: Rgba, opaque: bool) -> (Rgba, Option<f32>) {
        let metrics = self.metrics();
        let tint = Rgba {
            a: metrics.opacity,
            ..glass
        };
        crate::glass::material(tint, glass, metrics.blur, opaque)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateValue<T> {
    normal: T,
    hovered: Option<T>,
    pressed: Option<T>,
    focused: Option<T>,
    disabled: Option<T>,
}

impl<T> StateValue<T> {
    pub fn new(normal: T) -> Self {
        Self {
            normal,
            hovered: None,
            pressed: None,
            focused: None,
            disabled: None,
        }
    }

    pub fn hovered(mut self, value: T) -> Self {
        self.hovered = Some(value);
        self
    }

    pub fn pressed(mut self, value: T) -> Self {
        self.pressed = Some(value);
        self
    }

    pub fn focused(mut self, value: T) -> Self {
        self.focused = Some(value);
        self
    }

    pub fn disabled(mut self, value: T) -> Self {
        self.disabled = Some(value);
        self
    }

    pub(crate) fn resolve(&self, state: ControlState) -> &T {
        [
            (state.disabled, &self.disabled),
            (state.pressed, &self.pressed),
            (state.hovered, &self.hovered),
            (state.focused, &self.focused),
        ]
        .into_iter()
        .find_map(|(active, value)| value.as_ref().filter(|_| active))
        .unwrap_or(&self.normal)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Rectangle,
    RoundedRectangle {
        radius: f32,
        style: CornerStyle,
    },
    UnevenRoundedRectangle {
        top_leading: f32,
        bottom_leading: f32,
        bottom_trailing: f32,
        top_trailing: f32,
        style: CornerStyle,
    },
    Capsule,
    Circle,
}

impl Shape {
    pub(crate) fn corner_radius(
        self,
        width: f32,
        height: f32,
        direction: LayoutDirection,
    ) -> CornerRadius {
        let round = |radius: CornerRadius, style: CornerStyle| match style {
            CornerStyle::Continuous => crate::style::continuous(radius),
            CornerStyle::Circular => radius,
        };
        match self {
            Self::Rectangle => CornerRadius::new_all(0.0),
            Self::RoundedRectangle { radius, style } => round(CornerRadius::new_all(radius), style),
            Self::UnevenRoundedRectangle {
                top_leading,
                bottom_leading,
                bottom_trailing,
                top_trailing,
                style,
            } => {
                let (top_left, top_right, bottom_right, bottom_left) = match direction {
                    LayoutDirection::LeftToRight => {
                        (top_leading, top_trailing, bottom_trailing, bottom_leading)
                    }
                    LayoutDirection::RightToLeft => {
                        (top_trailing, top_leading, bottom_leading, bottom_trailing)
                    }
                };
                let radius = CornerRadius {
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                    smoothing: 0.0,
                };
                round(radius, style)
            }
            Self::Capsule | Self::Circle => CornerRadius::new_all(width.min(height) / 2.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CornerStyle {
    Continuous,
    Circular,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowStyle {
    pub color: Rgba,
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub inner: bool,
}

impl From<ctdesign::ShadowLayer> for ShadowStyle {
    fn from(layer: ctdesign::ShadowLayer) -> Self {
        Self {
            color: layer.color,
            x: 0.0,
            y: layer.y_offset as f32,
            blur: layer.blur as f32,
            spread: 0.0,
            inner: false,
        }
    }
}

impl From<ShadowStyle> for Shadow {
    fn from(style: ShadowStyle) -> Self {
        let shadow = Shadow::from((
            style.x,
            style.y,
            style.blur,
            style.spread,
            color(style.color),
        ));
        if style.inner { shadow.inset() } else { shadow }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Animation {
    None,
    Ease {
        duration_ms: u64,
    },
    Spring {
        duration_ms: u64,
        bounce: f32,
    },
    Custom {
        duration_ms: u64,
        curve: fn(f32) -> f32,
    },
}

impl Animation {
    pub(crate) fn duration_ms(self) -> u64 {
        match self {
            Self::None => 0,
            Self::Ease { duration_ms }
            | Self::Spring { duration_ms, .. }
            | Self::Custom { duration_ms, .. } => duration_ms,
        }
    }

    pub(crate) fn progress(self, elapsed: f32) -> f32 {
        let spring = |bounce| {
            crate::animation::Spring::new(0.0, 1.0, Duration::ZERO, bounce).value_at(elapsed)
        };
        match self {
            Self::None => 1.0,
            Self::Ease { .. } => spring(0.0),
            Self::Spring { bounce, .. } => spring(bounce),
            Self::Custom { curve, .. } => curve(elapsed),
        }
    }
}

pub type ColorEffect = fn(Rgba, ControlState) -> Rgba;

pub fn system_effect(color: Rgba, _: ControlState) -> Rgba {
    color
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Leading,
    Trailing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutDirection {
    LeftToRight,
    RightToLeft,
}

pub(crate) struct Paints {
    palette: Palette,
    backdrop: Option<Tinted>,
    glass: Rgba,
    opaque: bool,
    state: ControlState,
    effect: ColorEffect,
}

impl Paints {
    pub(crate) fn current(state: ControlState, effect: ColorEffect) -> Self {
        Self {
            palette: Palette::current(),
            backdrop: crate::backdrop::tinted().filter(|_| crate::platform::window_activated()),
            glass: crate::glass::glass_tint(),
            opaque: crate::glass::transparency_reduced(),
            state,
            effect,
        }
    }

    pub(crate) fn fill(&self, paint: &StateValue<ShapeStyle>) -> Fill {
        self.fill_of(paint.resolve(self.state))
    }

    pub(crate) fn blur(&self, paint: &StateValue<ShapeStyle>) -> Option<f32> {
        match paint.resolve(self.state) {
            ShapeStyle::Material(thickness) => thickness.material(self.glass, self.opaque).1,
            _ => None,
        }
    }

    pub(crate) fn color(&self, paint: &ShapeStyle) -> Rgba {
        (self.effect)(self.base(paint), self.state)
    }

    fn fill_of(&self, paint: &ShapeStyle) -> Fill {
        let ink = |rgba: Rgba| color((self.effect)(rgba, self.state));
        match (paint, self.backdrop) {
            (ShapeStyle::Gradient(gradient), _) => gradient
                .stops
                .iter()
                .fold(
                    LinearGradient::new().angle(gradient.angle),
                    |linear, (stop, position)| {
                        linear.stop((color(self.color(stop)), position * 100.0))
                    },
                )
                .into(),
            (ShapeStyle::Backdrop, Some(backdrop)) => {
                crate::backdrop::highlights(backdrop, ink).into()
            }
            _ => color(self.color(paint)).into(),
        }
    }

    fn base(&self, paint: &ShapeStyle) -> Rgba {
        match paint {
            ShapeStyle::Role(role) => role.color(&self.palette),
            ShapeStyle::Accent => self.palette.accent,
            ShapeStyle::Color(rgba) => *rgba,
            ShapeStyle::Gradient(gradient) => gradient
                .stops
                .first()
                .map_or(Rgba::TRANSPARENT, |(stop, _)| self.base(stop)),
            ShapeStyle::Backdrop => self
                .backdrop
                .map_or(self.palette.accent, |backdrop| backdrop.highlights()[0]),
            ShapeStyle::Material(thickness) => thickness.material(self.glass, self.opaque).0,
            ShapeStyle::Clear => Rgba::TRANSPARENT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: ControlState = ControlState {
        hovered: true,
        pressed: true,
        focused: true,
        disabled: true,
    };

    fn every_state() -> StateValue<&'static str> {
        StateValue::new("normal")
            .hovered("hovered")
            .pressed("pressed")
            .focused("focused")
            .disabled("disabled")
    }

    #[test]
    fn disabled_wins_then_pressed_then_hovered_then_focused() {
        let value = every_state();
        assert_eq!(*value.resolve(ALL), "disabled");
        let enabled = ControlState {
            disabled: false,
            ..ALL
        };
        assert_eq!(*value.resolve(enabled), "pressed");
        let released = ControlState {
            pressed: false,
            ..enabled
        };
        assert_eq!(*value.resolve(released), "hovered");
        let away = ControlState {
            hovered: false,
            ..released
        };
        assert_eq!(*value.resolve(away), "focused");
        assert_eq!(*value.resolve(ControlState::default()), "normal");
    }

    #[test]
    fn a_state_without_its_own_value_falls_back_to_the_next_one() {
        let value = StateValue::new("normal").hovered("hovered");
        assert_eq!(*value.resolve(ALL), "hovered");
        assert_eq!(*StateValue::new("normal").resolve(ALL), "normal");
    }

    #[test]
    fn reduced_transparency_makes_a_material_solid_and_sharp() {
        let glass = Rgba::rgb8(0x20, 0x30, 0x40);
        assert_eq!(Thickness::Thin.material(glass, true), (glass, None));
        let (tint, blur) = Thickness::Thin.material(glass, false);
        assert_eq!(tint.a, MATERIAL.thin.opacity);
        assert_eq!(blur, Some(MATERIAL.thin.blur));
    }
}
