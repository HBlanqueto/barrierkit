use ctdesign::{
    GLASS_INK_DARK, GLASS_INK_FAINT_DARK, GLASS_INK_FAINT_LIGHT, GLASS_INK_LIGHT,
    GLASS_INK_MUTED_DARK, GLASS_INK_MUTED_LIGHT, MENU_DARK_INK_ABOVE, Rgba,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuInk {
    pub fg: Rgba,
    pub shortcut: Rgba,
    pub disabled: Rgba,
}

const INK_ON_LIGHT_PANEL: MenuInk = MenuInk {
    fg: GLASS_INK_LIGHT,
    shortcut: GLASS_INK_MUTED_LIGHT,
    disabled: GLASS_INK_FAINT_LIGHT,
};

const INK_ON_DARK_PANEL: MenuInk = MenuInk {
    fg: GLASS_INK_DARK,
    shortcut: GLASS_INK_MUTED_DARK,
    disabled: GLASS_INK_FAINT_DARK,
};

pub fn panel_luma(panel: Rgba) -> f32 {
    panel.luma()
}

pub fn dark_ink_over(panel_luma: f32) -> bool {
    panel_luma > MENU_DARK_INK_ABOVE
}

pub fn ink_over(panel_luma: f32) -> MenuInk {
    if dark_ink_over(panel_luma) {
        INK_ON_LIGHT_PANEL
    } else {
        INK_ON_DARK_PANEL
    }
}
