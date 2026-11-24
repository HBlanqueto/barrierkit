use ctdesign::{
    MATERIAL_CONTENT_BACKGROUND_DARK, MATERIAL_CONTENT_BACKGROUND_LIGHT,
    MATERIAL_FULL_SCREEN_UI_DARK, MATERIAL_FULL_SCREEN_UI_LIGHT, MATERIAL_HEADER_VIEW_DARK,
    MATERIAL_HEADER_VIEW_LIGHT, MATERIAL_HUD_WINDOW_DARK, MATERIAL_HUD_WINDOW_LIGHT,
    MATERIAL_MENU_DARK, MATERIAL_MENU_LIGHT, MATERIAL_POPOVER_DARK, MATERIAL_POPOVER_LIGHT,
    MATERIAL_SELECTION_DARK, MATERIAL_SELECTION_LIGHT, MATERIAL_SHEET_DARK, MATERIAL_SHEET_LIGHT,
    MATERIAL_SIDEBAR_DARK, MATERIAL_SIDEBAR_LIGHT, MATERIAL_TITLEBAR_DARK, MATERIAL_TITLEBAR_LIGHT,
    MATERIAL_TOOL_TIP_DARK, MATERIAL_TOOL_TIP_LIGHT, MATERIAL_UNDER_PAGE_BACKGROUND_DARK,
    MATERIAL_UNDER_PAGE_BACKGROUND_LIGHT, MATERIAL_UNDER_WINDOW_BACKGROUND_DARK,
    MATERIAL_UNDER_WINDOW_BACKGROUND_LIGHT, MATERIAL_WINDOW_BACKGROUND_DARK,
    MATERIAL_WINDOW_BACKGROUND_LIGHT, MaterialLook, MaterialSet,
};

use crate::{ColorScheme, icon_style::luma};

const KERNEL_REACH: f32 = 3.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Titlebar,
    Selection,
    Menu,
    Popover,
    Sidebar,
    HeaderView,
    Sheet,
    WindowBackground,
    HudWindow,
    FullScreenUi,
    ToolTip,
    ContentBackground,
    UnderWindowBackground,
    UnderPageBackground,
}

pub fn looks(material: Material, scheme: ColorScheme) -> MaterialSet {
    let [light, dark] = match material {
        Material::Titlebar => [MATERIAL_TITLEBAR_LIGHT, MATERIAL_TITLEBAR_DARK],
        Material::Selection => [MATERIAL_SELECTION_LIGHT, MATERIAL_SELECTION_DARK],
        Material::Menu => [MATERIAL_MENU_LIGHT, MATERIAL_MENU_DARK],
        Material::Popover => [MATERIAL_POPOVER_LIGHT, MATERIAL_POPOVER_DARK],
        Material::Sidebar => [MATERIAL_SIDEBAR_LIGHT, MATERIAL_SIDEBAR_DARK],
        Material::HeaderView => [MATERIAL_HEADER_VIEW_LIGHT, MATERIAL_HEADER_VIEW_DARK],
        Material::Sheet => [MATERIAL_SHEET_LIGHT, MATERIAL_SHEET_DARK],
        Material::WindowBackground => [
            MATERIAL_WINDOW_BACKGROUND_LIGHT,
            MATERIAL_WINDOW_BACKGROUND_DARK,
        ],
        Material::HudWindow => [MATERIAL_HUD_WINDOW_LIGHT, MATERIAL_HUD_WINDOW_DARK],
        Material::FullScreenUi => [MATERIAL_FULL_SCREEN_UI_LIGHT, MATERIAL_FULL_SCREEN_UI_DARK],
        Material::ToolTip => [MATERIAL_TOOL_TIP_LIGHT, MATERIAL_TOOL_TIP_DARK],
        Material::ContentBackground => [
            MATERIAL_CONTENT_BACKGROUND_LIGHT,
            MATERIAL_CONTENT_BACKGROUND_DARK,
        ],
        Material::UnderWindowBackground => [
            MATERIAL_UNDER_WINDOW_BACKGROUND_LIGHT,
            MATERIAL_UNDER_WINDOW_BACKGROUND_DARK,
        ],
        Material::UnderPageBackground => [
            MATERIAL_UNDER_PAGE_BACKGROUND_LIGHT,
            MATERIAL_UNDER_PAGE_BACKGROUND_DARK,
        ],
    };
    scheme.pick([light, dark, dark])
}

fn kernel(sigma: f32) -> Vec<f32> {
    let reach = (sigma * KERNEL_REACH).ceil() as i32;
    let weights: Vec<f32> = (-reach..=reach)
        .map(|offset| (-0.5 * (offset as f32 / sigma).powi(2)).exp())
        .collect();
    let sum: f32 = weights.iter().sum();
    weights.into_iter().map(|weight| weight / sum).collect()
}

// macOS blurs with inputNormalizeEdges: a region sees only itself, so the weights are renormalised at its borders.
pub fn blur_region(line: &[f32], sigma: f32) -> Vec<f32> {
    if sigma <= 0.0 {
        return line.to_vec();
    }
    let weights = kernel(sigma);
    let reach = (weights.len() / 2) as i32;
    (0..line.len() as i32)
        .map(|at| {
            let (mut sum, mut total) = (0.0, 0.0);
            for (index, weight) in weights.iter().enumerate() {
                let from = at + index as i32 - reach;
                if let Some(value) = usize::try_from(from).ok().and_then(|from| line.get(from)) {
                    sum += weight * value;
                    total += weight;
                }
            }
            sum / total
        })
        .collect()
}

// The order is the layer stack's: saturate and clamp the backdrop, veil, then the cap (darken) and the floor (lighten).
pub fn over(look: &MaterialLook, backdrop: [f32; 3]) -> [f32; 3] {
    let luma = luma(backdrop);
    let veil = [look.veil.r, look.veil.g, look.veil.b];
    let mut shown = [0.0; 3];
    for (index, channel) in backdrop.iter().enumerate() {
        let saturated = (luma + look.saturation * (channel - luma)).clamp(0.0, 1.0);
        let veiled = saturated + (veil[index] - saturated) * look.veil.a;
        shown[index] =
            (look.gain * veiled.min(look.cap).max(look.floor) + look.bias).clamp(0.0, 1.0);
    }
    shown
}
