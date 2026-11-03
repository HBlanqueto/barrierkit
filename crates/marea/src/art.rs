use barrier_kit::theme::color;
use freya_core::prelude::*;

use crate::catalog::Title;

pub fn art_fill(title: &Title) -> LinearGradient {
    let (css_angle, from, to) = title.art;
    LinearGradient::new()
        .angle(css_angle + 180.0)
        .stop((color(from), 0.0))
        .stop((color(to), 100.0))
}

pub fn logo(title: &Title, scale: f32) -> Label {
    let logo = &title.logo;
    label()
        .text(title.name)
        .font_family(logo.family)
        .font_size(logo.size * scale)
        .font_weight(i32::from(logo.weight))
        .font_slant(if logo.italic {
            FontSlant::Italic
        } else {
            FontSlant::Upright
        })
        .color(color(logo.ink))
        .text_shadow(TextShadow {
            color: color(vow_design::Rgba::rgba8(0, 0, 0, 0.6)),
            offset: (0.0, 2.0),
            blur_sigma: 7.0,
        })
}
