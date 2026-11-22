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

use crate::ColorScheme;
use crate::icon_style::LUMA;

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
    let luma: f32 = backdrop
        .iter()
        .zip(LUMA)
        .map(|(channel, weight)| channel * weight)
        .sum();
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

#[cfg(test)]
mod tests {
    use ctdesign::{
        MATERIAL_BAR_DARK, MATERIAL_BAR_LIGHT, MATERIAL_REGULAR_DARK, MATERIAL_REGULAR_LIGHT,
        MATERIAL_THICK_DARK, MATERIAL_THICK_LIGHT, MATERIAL_THIN_DARK, MATERIAL_THIN_LIGHT,
        MATERIAL_ULTRA_THICK_DARK, MATERIAL_ULTRA_THICK_LIGHT, MATERIAL_ULTRA_THIN_DARK,
        MATERIAL_ULTRA_THIN_LIGHT,
    };

    use super::*;

    const ALL: [Material; 14] = [
        Material::Titlebar,
        Material::Selection,
        Material::Menu,
        Material::Popover,
        Material::Sidebar,
        Material::HeaderView,
        Material::Sheet,
        Material::WindowBackground,
        Material::HudWindow,
        Material::FullScreenUi,
        Material::ToolTip,
        Material::ContentBackground,
        Material::UnderWindowBackground,
        Material::UnderPageBackground,
    ];
    const SWIFTUI: [[MaterialSet; 2]; 6] = [
        [MATERIAL_ULTRA_THIN_LIGHT, MATERIAL_ULTRA_THIN_DARK],
        [MATERIAL_THIN_LIGHT, MATERIAL_THIN_DARK],
        [MATERIAL_REGULAR_LIGHT, MATERIAL_REGULAR_DARK],
        [MATERIAL_THICK_LIGHT, MATERIAL_THICK_DARK],
        [MATERIAL_ULTRA_THICK_LIGHT, MATERIAL_ULTRA_THICK_DARK],
        [MATERIAL_BAR_LIGHT, MATERIAL_BAR_DARK],
    ];
    const EDGE_AT: [i32; 7] = [-40, -20, -10, 0, 10, 20, 40];
    const SAMPLES: &str = include_str!("../tests/data/material-samples.csv");

    fn rows(kind: &'static str) -> impl Iterator<Item = (usize, bool, Vec<u8>)> {
        SAMPLES.lines().filter_map(move |line| {
            let mut fields = line.split(',');
            (fields.next()? == kind).then(|| {
                let numbers: Vec<u8> = fields.map(|field| field.parse().unwrap()).collect();
                (
                    usize::from(numbers[0]),
                    numbers[1] == 1,
                    numbers[2..].to_vec(),
                )
            })
        })
    }

    fn scheme(dark: bool) -> ColorScheme {
        if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        }
    }

    fn unit(levels: &[u8]) -> [f32; 3] {
        [levels[0], levels[1], levels[2]].map(|level| f32::from(level) / 255.0)
    }

    fn worst(shown: [f32; 3], measured: &[u8]) -> f32 {
        shown
            .iter()
            .zip(measured)
            .map(|(shown, measured)| (shown * 255.0 - f32::from(*measured)).abs())
            .fold(0.0, f32::max)
    }

    #[test]
    fn a_region_that_ends_at_the_step_does_not_see_beyond_it() {
        let line: Vec<f32> = (0..200).map(|x| if x < 100 { 1.0 } else { 0.0 }).collect();
        let blurred = blur_region(&line, 27.6);
        assert!(
            (blurred[0] - 1.0).abs() < 0.01,
            "white at the border is {}",
            blurred[0]
        );
        let lone: Vec<f32> = vec![0.7; 50];
        assert!(
            blur_region(&lone, 27.6)
                .iter()
                .all(|value| (value - 0.7).abs() < 1e-6)
        );
        assert!(
            (blurred[99] + blurred[100] - 1.0).abs() < 1e-3,
            "the step is not centred"
        );
    }

    #[test]
    fn an_inactive_material_never_blurs_and_is_opaque_but_for_the_sheet_and_selection() {
        for material in ALL {
            for dark in [false, true] {
                let inactive = looks(material, scheme(dark)).inactive;
                assert_eq!(
                    inactive.sigma, 0.0,
                    "{material:?} dark {dark} inactive blurs"
                );
                let may_be_translucent =
                    material == Material::Selection || (material == Material::Sheet && dark);
                assert_eq!(
                    inactive.veil.a < 1.0,
                    may_be_translucent,
                    "{material:?} dark {dark} inactive veil alpha {}",
                    inactive.veil.a
                );
            }
        }
    }

    #[test]
    fn the_materials_show_the_gray_and_colour_ramps_of_the_captures() {
        let ramp: Vec<f32> = (0..1024).map(|row| (row / 4) as f32 / 255.0).collect();
        let mut count = 0;
        for (index, dark, levels) in rows("ramp") {
            let material = ALL[index];
            let look = looks(material, scheme(dark)).active;
            let gray = levels[0] == levels[1] && levels[1] == levels[2];
            let behind = if gray {
                [blur_region(&ramp, look.sigma)[usize::from(levels[0]) * 4 + 2]; 3]
            } else {
                unit(&levels[..3])
            };
            let shown = over(&look, behind);
            let error = worst(shown, &levels[3..]);
            assert!(
                error <= 1.5,
                "{material:?} dark {dark} over {:?} shows {shown:?}, macOS {:?} ({error})",
                &levels[..3],
                &levels[3..]
            );
            count += 1;
        }
        assert_eq!(
            count,
            SAMPLES
                .lines()
                .filter(|line| line.starts_with("ramp"))
                .count()
        );
    }

    #[test]
    fn the_swiftui_materials_show_the_gray_ramps_of_the_captures() {
        let ramp: Vec<f32> = (0..1024).map(|row| (row / 4) as f32 / 255.0).collect();
        for (index, dark, levels) in rows("swiftui") {
            let look = SWIFTUI[index][usize::from(dark)].active;
            let behind = [blur_region(&ramp, look.sigma)[usize::from(levels[0]) * 4 + 2]; 3];
            let shown = over(&look, behind);
            let error = worst(shown, &levels[3..]);
            assert!(
                error <= 1.5,
                "SwiftUI material {index} dark {dark} over {:?} is {error} from {:?}",
                &levels[..3],
                &levels[3..]
            );
        }
    }

    #[test]
    fn the_swiftui_looks_of_reduced_transparency_are_solids_and_only_the_bar_goes_inactive() {
        for (index, dark, levels) in rows("swiftreduced") {
            let set = SWIFTUI[index][usize::from(dark)];
            for (look, measured) in [
                (set.reduced, &levels[..3]),
                (set.reduced_contrast, &levels[3..]),
            ] {
                assert_eq!(
                    look.sigma, 0.0,
                    "SwiftUI material {index} dark {dark} reduced blurs"
                );
                let error = worst(over(&look, [0.5; 3]), measured);
                assert!(
                    error <= 1.0,
                    "SwiftUI material {index} dark {dark} reduced is {error} from {measured:?}"
                );
            }
            assert_eq!(
                set.inactive.sigma == 0.0,
                index == 5,
                "SwiftUI material {index} dark {dark} inactive"
            );
        }
    }

    #[test]
    fn the_reduced_looks_are_the_solids_of_the_captures_over_any_backdrop() {
        for (index, dark, levels) in rows("reduced") {
            let material = ALL[index];
            let set = looks(material, scheme(dark));
            for (look, measured) in [
                (set.reduced, &levels[..3]),
                (set.reduced_contrast, &levels[3..]),
            ] {
                assert_eq!(look.sigma, 0.0, "{material:?} dark {dark} reduced blurs");
                for backdrop in [[0.0; 3], [0.5; 3], [1.0; 3]] {
                    let error = worst(over(&look, backdrop), measured);
                    assert!(
                        error <= 1.0,
                        "{material:?} dark {dark} reduced over {backdrop:?} is {error} from {measured:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_blurred_step_matches_the_captured_profile() {
        for (index, dark, levels) in rows("step") {
            let material = ALL[index];
            let look = looks(material, scheme(dark)).active;
            let line: Vec<f32> = (0..200)
                .map(|x| f32::from(if x < 100 { levels[0] } else { levels[1] }) / 255.0)
                .collect();
            let blurred = blur_region(&line, look.sigma);
            for (at, measured) in EDGE_AT.iter().zip(&levels[2..]) {
                let shown = over(&look, [blurred[(100 + at) as usize]; 3])[1] * 255.0;
                assert!(
                    (shown - f32::from(*measured)).abs() <= 2.0,
                    "{material:?} dark {dark} {at} px from the step shows {shown}, macOS {measured}"
                );
            }
        }
    }
}
