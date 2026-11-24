use ctappearance::ColorScheme;
use ctdesign::GLASS;

// How much of the lightness of what is behind the glass shows through it.
pub fn glass_lightness(scheme: ColorScheme) -> f32 {
    if scheme.is_dark() {
        GLASS.lightness_dark
    } else {
        GLASS.lightness_light
    }
}

// How much lighter the glass panel's border is than the fill under it, and how much more where the wallpaper shows.
pub fn glass_rim_lift(scheme: ColorScheme) -> (f32, f32) {
    if scheme.is_dark() {
        (GLASS.rim_lift_dark, GLASS.rim_lift_edge_dark)
    } else {
        (GLASS.rim_lift_light, GLASS.rim_lift_edge_light)
    }
}

// How many times the glass blurs what is behind it.
pub fn glass_blur_passes(scheme: ColorScheme) -> u32 {
    if scheme.is_dark() {
        GLASS.blur_passes_dark
    } else {
        GLASS.blur_passes_light
    }
}

// The wallpaper shows along the left, top and bottom edges of a panel: `intensity` at the edge, fading over `span` logical pixels, with `chroma` and `lightness` of what is `offset` logical pixels outside the window (docs/barrierkit/10-sidebar.md#glass).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassEdge {
    pub intensity: f32,
    pub span: f32,
    pub chroma: f32,
    pub lightness: f32,
    pub cap: f32,
    pub offset: f32,
}

pub fn glass_edge(scheme: ColorScheme) -> GlassEdge {
    if scheme.is_dark() {
        GlassEdge {
            intensity: GLASS.edge_intensity_dark,
            span: GLASS.edge_span_dark,
            chroma: GLASS.edge_chroma_dark,
            lightness: GLASS.edge_lightness_dark,
            cap: GLASS.edge_cap_dark,
            offset: GLASS.edge_offset,
        }
    } else {
        GlassEdge {
            intensity: GLASS.edge_intensity_light,
            span: GLASS.edge_span_light,
            chroma: GLASS.edge_chroma_light,
            lightness: GLASS.edge_lightness_light,
            cap: GLASS.edge_cap_light,
            offset: GLASS.edge_offset,
        }
    }
}

// The glass panel's shadow on the window around it: `opacity` next to the panel, fading over `span` logical pixels, as dark as `lightness` and with `tint` of the wallpaper's chroma.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassShadow {
    pub opacity: f32,
    pub span: f32,
    pub tint: f32,
    pub lightness: f32,
}

pub fn glass_shadow(scheme: ColorScheme) -> Option<GlassShadow> {
    (!scheme.is_dark()).then_some(GlassShadow {
        opacity: GLASS.shadow_opacity,
        span: GLASS.shadow_span,
        tint: GLASS.shadow_tint,
        lightness: GLASS.shadow_lightness,
    })
}
