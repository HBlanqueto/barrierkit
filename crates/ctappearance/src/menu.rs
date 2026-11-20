use ctdesign::{
    DockSurface, GLASS_INK_DARK, GLASS_INK_FAINT_DARK, GLASS_INK_FAINT_LIGHT, GLASS_INK_LIGHT,
    GLASS_INK_MUTED_DARK, GLASS_INK_MUTED_LIGHT, MENU_DARK_INK_ABOVE, Rgba,
};

use crate::backdrop::Oklab;
use crate::icon_style::LUMA;

const CAP_SOFTNESS: f32 = 0.1;

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

fn smooth_union(a: f32, b: f32, softness: f32) -> f32 {
    let blend = (softness - (a - b).abs()).max(0.0) / softness;
    a.min(b) - blend * blend * softness * 0.25
}

pub fn panel_over(glass: &DockSurface, behind: Rgba) -> Rgba {
    let seen = behind.mix(glass.glass, glass.glass.a);
    let Some(tone) = glass.tone else {
        return seen;
    };
    let lifted = Oklab::from_rgba(seen).l + tone.lift;
    let lightness = smooth_union(lifted, tone.cap, CAP_SOFTNESS);
    let chroma = tone.chroma * (lightness / lifted.max(0.001)).min(1.0);
    let behind = Oklab::from_rgba(behind);
    Oklab {
        l: lightness,
        a: behind.a * chroma,
        b: behind.b * chroma,
    }
    .to_rgba()
}

pub fn panel_luma(panel: Rgba) -> f32 {
    let [r, g, b] = LUMA;
    r * panel.r + g * panel.g + b * panel.b
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_checker_panel_at_luma_0_53_takes_dark_ink() {
        assert!(dark_ink_over(0.53));
        assert_eq!(ink_over(0.53), INK_ON_LIGHT_PANEL);
    }

    #[test]
    fn a_black_panel_at_luma_0_17_takes_light_ink() {
        assert!(!dark_ink_over(0.17));
        assert_eq!(ink_over(0.17), INK_ON_DARK_PANEL);
    }

    #[test]
    fn the_luma_of_a_grey_is_its_value() {
        let grey = Rgba::rgb8(0x8d, 0x8d, 0x8d);
        assert!((panel_luma(grey) - grey.r).abs() < 1e-5);
    }
}
