use super::ControlStyle;
use crate::theme::{SurfaceKind, menu_palette, popover_palette, resolved};

pub type MenuStyle = ctdesign::MenuPalette;

impl ControlStyle for MenuStyle {
    fn tengen() -> Self {
        menu_palette(&resolved(), SurfaceKind::Toplevel)
    }
}

pub(crate) fn popover_style() -> MenuStyle {
    super::resolve_from(
        popover_palette(&resolved(), SurfaceKind::Toplevel),
        &super::Theme::tengen(),
    )
}
