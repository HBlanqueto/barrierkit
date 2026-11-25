use ctdesign::{ICON_STYLE, Rgba};
use serde::{Deserialize, Serialize};

use crate::backdrop::Oklab;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IconStyle {
    #[default]
    Default,
    Dark,
    Clear,
    // A monochrome icon is a tinted one in graphite; `load_appearance` gives it that color.
    #[serde(alias = "monochrome")]
    Tinted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IconTone {
    Light,
    Dark,
    #[default]
    Automatic,
}

impl IconStyle {
    pub fn dark(self, tone: IconTone, scheme_dark: bool) -> bool {
        self != Self::Default
            && match tone {
                IconTone::Light => false,
                IconTone::Dark => true,
                IconTone::Automatic => scheme_dark,
            }
    }

    /// Row-major 4×5 color matrix on straight-alpha colors in 0..1; `None` keeps the colors.
    pub fn matrix(self, tint: Rgba) -> Option<[f32; 20]> {
        match self {
            Self::Default | Self::Dark => None,
            Self::Clear => {
                let floor = ICON_STYLE.clear_floor;
                let mut matrix = duotone(gray(floor), gray(floor + ICON_STYLE.clear_range));
                let gain = ICON_STYLE.clear_alpha_gain;
                matrix[15..18].copy_from_slice(&Rgba::LUMA.map(|weight| weight * gain));
                matrix[18] = 0.0;
                Some(matrix)
            }
            Self::Tinted => {
                let (dark, light) = tint_ends(tint);
                Some(duotone(dark, light))
            }
        }
    }

    /// As `matrix`, but a clear icon also loses its dark body: its alpha starts at `ICON_STYLE.clear_cut`, for a compositor that draws the plate as glass.
    pub fn matrix_on_glass(self, tint: Rgba) -> Option<[f32; 20]> {
        let mut matrix = self.matrix(tint)?;
        if self == Self::Clear {
            matrix[19] = -ICON_STYLE.clear_alpha_gain * ICON_STYLE.clear_cut;
        }
        Some(matrix)
    }

    /// The matrix makes the alpha follow the luminance, so the result is also cut to the image's own shape.
    pub fn luminance_to_alpha(self) -> bool {
        self == Self::Clear
    }
}

pub(crate) fn gray(value: f32) -> Rgba {
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
        let [r, g, b] = Rgba::LUMA.map(|weight| weight * (light - dark));
        [r, g, b, 0.0, dark]
    };
    let rows = [
        row(dark.r, light.r),
        row(dark.g, light.g),
        row(dark.b, light.b),
        [0.0, 0.0, 0.0, 1.0, 0.0],
    ];
    std::array::from_fn(|i| rows[i / 5][i % 5])
}

fn tint_ends(tint: Rgba) -> (Rgba, Rgba) {
    let tint = Oklab::from_rgba(tint);
    let end = |lightness: f32, chroma: f32| {
        Oklab {
            l: lightness,
            a: tint.a * chroma,
            b: tint.b * chroma,
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
