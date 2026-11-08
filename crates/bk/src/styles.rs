use std::time::Duration;

pub use about_view::AboutViewStyle;
pub use alert::AlertStyle;
pub use collection_view::CollectionViewStyle;
pub use content_unavailable_view::ContentUnavailableViewStyle;
use ctappearance::backdrop::Backdrop as Tinted;
pub use ctdesign::Rgba;
use ctdesign::{MATERIAL, MaterialThickness};
use freya_core::prelude::*;

use crate::backdrop::Tint;
use crate::style::Palette;
use crate::theme::color;

mod about_view;
mod alert;
mod avatar;
mod backdrop;
mod badge;
mod button;
mod capacity_indicator;
#[cfg(feature = "charts")]
mod chart;
mod checkbox;
mod choice_cards;
mod collection_view;
mod color_picker;
mod color_well;
mod combo_box;
mod combo_button;
mod content_unavailable_view;
mod control_group;
mod date_picker;
mod disclosure_group;
mod divider;
mod floating_pane;
mod form;
mod fullscreen_bar;
mod gauge;
mod glass_container;
mod group_box;
mod help_link;
mod icon_button;
mod image;
mod image_well;
mod level_slider;
mod link;
mod material;
mod media_button;
mod menu;
mod menu_button;
mod outline_view;
mod path_control;
mod popup_button;
#[cfg(feature = "print")]
mod print_preview;
mod progress_ring;
mod progress_view;
mod radio_group;
mod rating_indicator;
mod relevancy_indicator;
mod round_button;
mod rule_editor;
mod scroll_view;
mod search_field;
mod secure_field;
mod segmented_control;
mod settings_window;
mod sheet;
mod sidebar;
mod sidebar_account;
mod sidebar_item;
mod sidebar_section;
mod slider;
mod spinner;
mod split_panes;
mod split_view;
mod stepper;
mod switch;
mod symbol_tile;
mod tab_view;
mod table;
mod text;
mod text_editor;
mod text_field;
mod text_view;
mod theme;
mod titlebar;
mod toggle_group;
mod token_field;
mod tool_group;
mod toolbar_search;
mod top_glass;
mod traffic_lights;
#[cfg(feature = "video")]
mod video_view;
mod window_chrome;
mod window_tabs;

pub use avatar::AvatarStyle;
pub use backdrop::BackdropStyle;
pub use badge::BadgeStyle;
pub use button::ButtonStyle;
pub use capacity_indicator::CapacityIndicatorStyle;
#[cfg(feature = "charts")]
pub use chart::ChartStyle;
pub use checkbox::CheckboxStyle;
pub use choice_cards::ChoiceCardsStyle;
pub use color_picker::ColorPickerStyle;
pub use color_well::ColorWellStyle;
pub use combo_box::ComboBoxStyle;
pub use combo_button::ComboButtonStyle;
pub use control_group::ControlGroupStyle;
pub use date_picker::DatePickerStyle;
pub use disclosure_group::DisclosureGroupStyle;
pub use divider::DividerStyle;
pub use floating_pane::FloatingPaneStyle;
pub use form::FormStyle;
pub use fullscreen_bar::FullscreenBarStyle;
pub use gauge::GaugeStyle;
pub use glass_container::GlassContainerStyle;
pub use group_box::GroupBoxStyle;
pub use help_link::HelpLinkStyle;
pub use icon_button::IconButtonStyle;
pub use image::ImageStyle;
pub use image_well::ImageWellStyle;
pub use level_slider::LevelSliderStyle;
pub use link::LinkStyle;
pub use material::MaterialStyle;
pub use media_button::MediaButtonStyle;
pub use menu::MenuStyle;
pub(crate) use menu::popover_style;
pub use menu_button::MenuButtonStyle;
pub use outline_view::OutlineViewStyle;
pub use path_control::PathControlStyle;
pub use popup_button::PopUpButtonStyle;
#[cfg(feature = "print")]
pub use print_preview::PrintPreviewStyle;
pub use progress_ring::ProgressRingStyle;
pub use progress_view::ProgressViewStyle;
pub use radio_group::RadioGroupStyle;
pub use rating_indicator::RatingIndicatorStyle;
pub use relevancy_indicator::RelevancyIndicatorStyle;
pub use round_button::RoundButtonStyle;
pub use rule_editor::RuleEditorStyle;
pub use scroll_view::ScrollViewStyle;
pub use search_field::SearchFieldStyle;
pub use secure_field::SecureFieldStyle;
pub use segmented_control::SegmentedControlStyle;
pub use settings_window::SettingsWindowStyle;
pub use sheet::SheetStyle;
pub use sidebar::SidebarStyle;
pub use sidebar_account::SidebarAccountStyle;
pub use sidebar_item::SidebarItemStyle;
pub use sidebar_section::SidebarSectionStyle;
pub use slider::SliderStyle;
pub use spinner::SpinnerStyle;
pub use split_panes::SplitPanesStyle;
pub use split_view::SplitViewStyle;
pub use stepper::StepperStyle;
pub use switch::SwitchStyle;
pub use symbol_tile::SymbolTileStyle;
pub use tab_view::TabViewStyle;
pub use table::TableStyle;
pub use text::TextStyle;
pub use text_editor::TextEditorStyle;
pub use text_field::TextFieldStyle;
pub use text_view::TextViewStyle;
pub use theme::{ControlStyle, StyleScope, Theme};
pub(crate) use theme::{
    PopupRoot, current_theme, reduce_motion, resolve, resolve_from, show_borders, use_provide_theme,
};
pub use titlebar::TitlebarStyle;
pub use toggle_group::ToggleGroupStyle;
pub use token_field::TokenFieldStyle;
pub use tool_group::ToolGroupStyle;
pub use toolbar_search::ToolbarSearchStyle;
pub use top_glass::TopGlassStyle;
pub use traffic_lights::TrafficLightsStyle;
#[cfg(feature = "video")]
pub use video_view::VideoViewStyle;
pub use window_chrome::WindowChromeStyle;
pub use window_tabs::WindowTabsStyle;

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
                let radius = CornerRadius::new(top_left, top_right, bottom_right, bottom_left);
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

pub(crate) trait Painted: BackgroundExt + DecorationExt + EffectExt + Sized {
    fn painted(self, paints: &Paints, paint: &StateValue<ShapeStyle>) -> Self {
        let view = self.background(paints.fill(paint));
        match paints.blur(paint) {
            Some(blur) => view.backdrop_blur(blur),
            None => view,
        }
    }

    fn bordered(self, paints: &Paints, border: Option<&(ShapeStyle, f32)>) -> Self {
        match border {
            Some(border) => self.border(crate::controls::inner_border(paints, border)),
            None => self,
        }
    }
}

impl<T: BackgroundExt + DecorationExt + EffectExt> Painted for T {}

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
