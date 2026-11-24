use ctdesign::{ACCENT_RULE, ACCENTS, Accent, AccentColors, OklchShift, Rgba};

use crate::backdrop::Oklab;

pub fn named(name: &str) -> Option<&'static Accent> {
    ACCENTS.iter().find(|accent| accent.name == name)
}

// Settings wrote a swatch's light color as `accent_color` before accents had names.
pub fn named_by_color(color: Rgba) -> Option<&'static Accent> {
    const ROUNDING: f32 = 0.5 / 255.0 + 0.001;
    ACCENTS.iter().find(|accent| {
        let control = accent.light.control;
        (control.r - color.r).abs() <= ROUNDING
            && (control.g - color.g).abs() <= ROUNDING
            && (control.b - color.b).abs() <= ROUNDING
    })
}

pub fn blue() -> &'static Accent {
    named("blue").expect("ctdesign has a blue accent")
}

pub(crate) fn graphite() -> &'static Accent {
    named("graphite").expect("ctdesign has a graphite accent")
}

pub fn derived(control: Rgba, dark: bool, contrast: bool) -> AccentColors {
    let rule = ACCENT_RULE;
    let blue = blue().colors(dark, contrast);
    let look = usize::from(dark) + 2 * usize::from(contrast);
    let toward_ink = |mix: [f32; 4]| {
        if dark {
            control.lighten(mix[look])
        } else {
            control.darken(mix[look])
        }
    };
    let selected_content = if contrast {
        control
    } else {
        shifted(
            control,
            pick(
                dark,
                rule.selected_content_light,
                rule.selected_content_dark,
            ),
        )
    };
    let focus_ring = if contrast {
        shifted(
            control,
            pick(
                dark,
                rule.focus_ring_contrast_light,
                rule.focus_ring_contrast_dark,
            ),
        )
    } else {
        shifted(
            control,
            pick(dark, rule.focus_ring_light, rule.focus_ring_dark),
        )
    };
    AccentColors {
        control,
        selected_content,
        selected_text: pick(
            dark,
            rule.selected_text_base_light,
            rule.selected_text_base_dark,
        )
        .mix(control, rule.selected_text_mix),
        focus_ring: Rgba {
            a: blue.focus_ring.a,
            ..focus_ring
        },
        toggle: toward_ink(rule.toggle_mix),
        toggle_disabled: toward_ink(rule.toggle_disabled_mix),
        ring: control,
        pressed: toward_ink(rule.pressed_mix),
        pressed_toggle: toward_ink(rule.pressed_toggle_mix),
        segment_disabled: Rgba {
            a: blue.segment_disabled.a,
            ..toward_ink(rule.segment_disabled_mix)
        },
        sidebar_selection: control,
    }
}

fn pick<T>(dark: bool, light: T, dark_: T) -> T {
    if dark { dark_ } else { light }
}

fn shifted(color: Rgba, shift: OklchShift) -> Rgba {
    let lab = Oklab::from_rgba(color);
    let chroma = lab.a.hypot(lab.b) * shift.chroma_scale;
    let hue = lab.b.atan2(lab.a) + shift.hue_shift;
    Rgba {
        a: color.a,
        ..Oklab {
            l: lab.l * shift.lightness_scale + shift.lightness_offset,
            a: chroma * hue.cos(),
            b: chroma * hue.sin(),
        }
        .to_rgba()
    }
}
