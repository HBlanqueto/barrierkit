use ctdesign::{HAIRLINE, MEDIA};

use crate::styles::{ControlStyle, ShapeStyle, StateValue};

/// States: only `normal`. Without compositor glass, or when the user reduces transparency,
/// the material and the top glass become their `tint`, opaque, and the glass container its
/// `solid`. The corner radius of `Material::new` and the radii of
/// `GlassContainer::item` stay arguments: they follow the content.
#[derive(Clone, Debug)]
pub struct MaterialStyle {
    /// Tengen: `Color(MEDIA.material_tint)`.
    pub tint: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color(MEDIA.material_rim), HAIRLINE))`, drawn inside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `Material::HudWindow`. The material a compositor that offers `limitless_effects_v1`
    /// draws behind `Material`; the kit then ignores `tint` and `border`.
    pub material: ctappearance::material::Material,
}

impl ControlStyle for MaterialStyle {
    fn tengen() -> Self {
        Self {
            tint: StateValue::new(ShapeStyle::Color(MEDIA.material_tint)),
            border: Some((ShapeStyle::Color(MEDIA.material_rim), HAIRLINE)),
            material: ctappearance::material::Material::HudWindow,
        }
    }
}
