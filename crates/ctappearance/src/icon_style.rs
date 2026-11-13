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
        let row = |scale: f32, offset: f32| {
            [
                LUMA[0] * scale,
                LUMA[1] * scale,
                LUMA[2] * scale,
                0.0,
                offset,
            ]
        };
        let keep_alpha = [0.0, 0.0, 0.0, 1.0, 0.0];
        let [r, g, b] = match self {
            Self::Default => return None,
            Self::Monochrome => [row(1.0, 0.0); 3],
            Self::Clear => [row(ICON_STYLE.clear_range, ICON_STYLE.clear_floor); 3],
            Self::Tinted => {
                let (dark, light) = tint_ends(accent);
                [
                    row(light.r - dark.r, dark.r),
                    row(light.g - dark.g, dark.g),
                    row(light.b - dark.b, dark.b),
                ]
            }
        };
        let alpha = if self.luminance_to_alpha() {
            row(ICON_STYLE.clear_alpha_gain, 0.0)
        } else {
            keep_alpha
        };
        let rows = [r, g, b, alpha];
        Some(std::array::from_fn(|i| rows[i / 5][i % 5]))
    }

    /// The matrix makes the alpha follow the luminance, so the result is also cut to the image's own shape.
    pub fn luminance_to_alpha(self) -> bool {
        self == Self::Clear
    }
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
