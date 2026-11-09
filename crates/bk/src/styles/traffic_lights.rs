use ctappearance::palette;
use ctappearance::traffic::{self, ButtonVisual, TrafficInput};
use ctdesign::{Rgba, TITLEBAR, TRAFFIC_GLYPHS, TRAFFIC_RIM_WIDTH};

use super::button::rounded;
use super::{Axis, ColorEffect, ControlStyle, Shape, ShapeStyle, StateValue, system_effect};

/// `fills`, `rims` and `glyphs` hold one paint per light, in the order close,
/// minimize, zoom.
///
/// States: hovered (the pointer is over the group), pressed (that light), focused
/// (the window is focused), disabled (zoom without zoom support, the other
/// lights with `.close_only(true)`). `.round(true)` is an instance override of
/// `shape` (`Circle`), `.vertical(true)` of `axis`. `Titlebar`, the rail and the
/// sidebar place their lights with `TrafficLights::for_window`, so their size
/// comes from the theme and the containers.
#[derive(Clone, Debug)]
pub struct TrafficLightsStyle {
    /// Tengen: `TITLEBAR.buttons.diameter`.
    pub diameter: f32,
    /// Tengen: `TITLEBAR.buttons.spacing`.
    pub spacing: f32,
    /// Tengen: `TRAFFIC_GLYPHS.box_size`.
    pub glyph_size: f32,
    /// Tengen: `TRAFFIC_RIM_WIDTH`.
    pub rim_width: f32,
    /// Tengen: `RoundedRectangle { radius: TITLEBAR.buttons.corner_radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen, with `rims`, `glyphs`: the `Color`s of `ctappearance::traffic::visual` for the
    /// user's `traffic_lights`: normal and disabled are the gray idle light without a glyph,
    /// focused the lit light, hovered the lit light with its glyph, pressed the darkened light.
    /// `glyphs` is `Clear` where no glyph shows.
    pub fills: [StateValue<ShapeStyle>; 3],
    /// Tengen, with `fills`, `glyphs`: the `Color`s of `ctappearance::traffic::visual` for the
    /// user's `traffic_lights`: normal and disabled are the gray idle light without a glyph,
    /// focused the lit light, hovered the lit light with its glyph, pressed the darkened light.
    /// `glyphs` is `Clear` where no glyph shows.
    pub rims: [StateValue<ShapeStyle>; 3],
    /// Tengen, with `fills`, `rims`: the `Color`s of `ctappearance::traffic::visual` for the
    /// user's `traffic_lights`: normal and disabled are the gray idle light without a glyph,
    /// focused the lit light, hovered the lit light with its glyph, pressed the darkened light.
    /// `glyphs` is `Clear` where no glyph shows.
    pub glyphs: [StateValue<ShapeStyle>; 3],
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Horizontal`.
    pub axis: Axis,
}

impl TrafficLightsStyle {
    pub(crate) fn length(&self) -> f32 {
        let count = TITLEBAR.buttons.order.len() as f32;
        count * self.diameter + (count - 1.0) * self.spacing
    }
}

fn paint(color: Option<Rgba>) -> ShapeStyle {
    color.map_or(ShapeStyle::Clear, ShapeStyle::Color)
}

impl ControlStyle for TrafficLightsStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let base = palette(resolved.scheme);
        let (inactive, rim) = (base.traffic_lights_inactive, base.traffic_rim);
        let looks = |group_hovered, pressed| {
            traffic::visual(
                resolved.lights,
                resolved.traffic_style,
                inactive,
                rim,
                TrafficInput {
                    focused: true,
                    group_hovered,
                    pressed,
                    zoom_supported: true,
                },
            )
        };
        let (lit, hovered) = (looks(false, None), looks(true, None));
        let idle = ButtonVisual::idle(inactive, rim);
        let by_light = |part: fn(&ButtonVisual) -> Option<Rgba>, press: bool| {
            std::array::from_fn(|index| {
                let kind = TITLEBAR.buttons.order[index].kind;
                let value = StateValue::new(paint(part(&idle)))
                    .focused(paint(part(&lit[index])))
                    .hovered(paint(part(&hovered[index])))
                    .disabled(paint(part(&idle)));
                if press {
                    value.pressed(paint(part(&looks(false, Some(kind))[index])))
                } else {
                    value
                }
            })
        };
        let diameter = TITLEBAR.buttons.diameter as f32;
        Self {
            diameter,
            spacing: TITLEBAR.buttons.spacing as f32,
            glyph_size: TRAFFIC_GLYPHS.box_size,
            rim_width: TRAFFIC_RIM_WIDTH,
            shape: rounded(resolved.traffic_shape.corner_radius(diameter)),
            fills: by_light(|visual| Some(visual.fill), true),
            rims: by_light(|visual| Some(visual.rim), true),
            glyphs: std::array::from_fn(|index| {
                let focused = if resolved.differentiate_without_color {
                    &hovered[index]
                } else {
                    &lit[index]
                };
                StateValue::new(paint(idle.glyph))
                    .focused(paint(focused.glyph))
                    .hovered(paint(hovered[index].glyph))
                    .disabled(paint(idle.glyph))
            }),
            opacity: 1.0,
            effect: system_effect,
            axis: Axis::Horizontal,
        }
    }
}
