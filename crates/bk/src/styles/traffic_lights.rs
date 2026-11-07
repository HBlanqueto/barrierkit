use ctappearance::traffic::{self, ButtonVisual, TrafficInput};
use ctappearance::{Resolved, TrafficLightShape, palette};
use ctdesign::{Rgba, TITLEBAR, TRAFFIC_GLYPHS, TRAFFIC_RIM_WIDTH};

use super::button::rounded;
use super::{Axis, ColorEffect, ControlStyle, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct TrafficLightsStyle {
    pub diameter: f32,
    pub spacing: f32,
    pub glyph_size: f32,
    pub rim_width: f32,
    pub shape: Shape,
    pub fills: [StateValue<ShapeStyle>; 3],
    pub rims: [StateValue<ShapeStyle>; 3],
    pub glyphs: [StateValue<ShapeStyle>; 3],
    pub opacity: f32,
    pub effect: ColorEffect,
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
            shape: rounded(TrafficLightShape::Square.corner_radius(diameter)),
            fills: by_light(|visual| Some(visual.fill), true),
            rims: by_light(|visual| Some(visual.rim), true),
            glyphs: by_light(|visual| visual.glyph, false),
            opacity: 1.0,
            effect: system_effect,
            axis: Axis::Horizontal,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}
