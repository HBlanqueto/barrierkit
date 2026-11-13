use ctdesign::{ICON_STYLE, Rgba};
use serde::{Deserialize, Serialize};

use crate::backdrop::Oklab;

const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IconStyle {
    #[default]
    Default,
    Monochrome,
    Clear,
    Tinted,
}

impl IconStyle {
    /// Row-major 4×5 color matrix on straight-alpha colors in 0..1; `None` keeps the colors.
    pub fn matrix(self, accent: Rgba) -> Option<[f32; 20]> {
        match self {
            Self::Default => None,
            Self::Monochrome => Some(duotone(gray(0.0), gray(1.0))),
            Self::Clear => {
                let floor = ICON_STYLE.clear_floor;
                let mut matrix = duotone(gray(floor), gray(floor + ICON_STYLE.clear_range));
                let gain = ICON_STYLE.clear_alpha_gain;
                matrix[15..18].copy_from_slice(&LUMA.map(|weight| weight * gain));
                matrix[18] = 0.0;
                Some(matrix)
            }
            Self::Tinted => {
                let (dark, light) = tint_ends(accent);
                Some(duotone(dark, light))
            }
        }
    }

    /// The matrix makes the alpha follow the luminance, so the result is also cut to the image's own shape.
    pub fn luminance_to_alpha(self) -> bool {
        self == Self::Clear
    }
}

fn gray(value: f32) -> Rgba {
    Rgba {
        r: value,
        g: value,
        b: value,
        a: 1.0,
    }
}

/// Black becomes `dark`, white becomes `light` and the alpha stays.
pub(crate) fn duotone(dark: Rgba, light: Rgba) -> [f32; 20] {
    let row = |dark: f32, light: f32| {
        [
            LUMA[0] * (light - dark),
            LUMA[1] * (light - dark),
            LUMA[2] * (light - dark),
            0.0,
            dark,
        ]
    };
    let rows = [
        row(dark.r, light.r),
        row(dark.g, light.g),
        row(dark.b, light.b),
        [0.0, 0.0, 0.0, 1.0, 0.0],
    ];
    std::array::from_fn(|i| rows[i / 5][i % 5])
}

fn tint_ends(accent: Rgba) -> (Rgba, Rgba) {
    let accent = Oklab::from_rgba(accent);
    let end = |lightness: f32, chroma: f32| {
        Oklab {
            l: lightness,
            a: accent.a * chroma,
            b: accent.b * chroma,
        }
        .to_rgba()
    };
    (
        end(ICON_STYLE.tint_dark_lightness, ICON_STYLE.tint_dark_chroma),
        end(
            ICON_STYLE.tint_light_lightness,
            ICON_STYLE.tint_light_chroma,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const AMBER: Rgba = Rgba::rgb8(0x8a, 0x5e, 0x1e);

    fn apply(matrix: [f32; 20], [r, g, b, a]: [f32; 4]) -> [f32; 4] {
        [0, 1, 2, 3].map(|row| {
            let m = &matrix[row * 5..row * 5 + 5];
            (m[0] * r + m[1] * g + m[2] * b + m[3] * a + m[4]).clamp(0.0, 1.0)
        })
    }

    #[test]
    fn default_keeps_the_colors() {
        assert_eq!(IconStyle::Default.matrix(AMBER), None);
    }

    #[test]
    fn monochrome_turns_red_into_an_equal_gray_with_the_same_alpha() {
        let [r, g, b, a] = apply(
            IconStyle::Monochrome.matrix(AMBER).unwrap(),
            [1.0, 0.0, 0.0, 0.5],
        );
        assert!((r - g).abs() < 1e-6 && (g - b).abs() < 1e-6 && (r - 0.2126).abs() < 1e-4);
        assert_eq!(a, 0.5);
    }

    #[test]
    fn tinted_sends_black_to_a_dark_accent_and_white_to_a_light_one() {
        let matrix = IconStyle::Tinted.matrix(AMBER).unwrap();
        let [dr, dg, db, _] = apply(matrix, [0.0, 0.0, 0.0, 1.0]);
        let [lr, lg, lb, _] = apply(matrix, [1.0, 1.0, 1.0, 1.0]);
        assert!(dr > db && lr > lb, "both ends keep the accent's warm hue");
        assert!(dr + dg + db < 0.8 && lr + lg + lb > 2.4);
    }

    #[test]
    fn clear_keeps_a_transparent_pixel_transparent_and_a_white_one_opaque() {
        let matrix = IconStyle::Clear.matrix(AMBER).unwrap();
        assert_eq!(apply(matrix, [0.0, 0.0, 0.0, 0.0])[3], 0.0);
        let [r, _, _, a] = apply(matrix, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(a, 1.0);
        assert!(r > 0.95);
        assert!(IconStyle::Clear.luminance_to_alpha());
    }
}
