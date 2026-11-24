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
