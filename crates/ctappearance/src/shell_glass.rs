use ctdesign::{Rgba, SHELL_GLASS_CHROMA_KNOTS, SHELL_GLASS_LEVELS, ShellGlass};

use crate::icon_style::LUMA;

fn linear(knots: &[f32], values: &[f32], at: f32) -> f32 {
    let last = knots.len() - 1;
    if at <= knots[0] {
        return values[0];
    }
    for step in 1..=last {
        if at <= knots[step] {
            let share = (at - knots[step - 1]) / (knots[step] - knots[step - 1]);
            return values[step - 1] + (values[step] - values[step - 1]) * share;
        }
    }
    values[last]
}

pub fn tone(glass: &ShellGlass, luma: f32) -> f32 {
    linear(&SHELL_GLASS_LEVELS, &glass.tone, luma)
}

pub fn chroma(glass: &ShellGlass, luma: f32) -> f32 {
    linear(&SHELL_GLASS_CHROMA_KNOTS, &glass.chroma, luma)
}

pub fn over(glass: &ShellGlass, blurred: Rgba) -> Rgba {
    let [r, g, b] = LUMA;
    let luma = r * blurred.r + g * blurred.g + b * blurred.b;
    let (tone, gain) = (tone(glass, luma), chroma(glass, luma));
    let channel = |value: f32| tone + gain * (value - luma);
    Rgba {
        r: channel(blurred.r),
        g: channel(blurred.g),
        b: channel(blurred.b),
        a: blurred.a,
    }
}

// Abramowitz and Stegun 7.1.26, absolute error below 1.5e-7.
fn erf(x: f32) -> f32 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let poly = ((((1.061_405_4 * t - 1.453_152_1) * t + 1.421_413_7) * t - 0.284_496_72) * t
        + 0.254_829_6)
        * t;
    (1.0 - poly * (-x * x).exp()).copysign(x)
}

fn gaussian_step(sigma: f32, distance: f32) -> f32 {
    0.5 * (1.0 + erf(distance / (sigma * std::f32::consts::SQRT_2)))
}

pub fn blurred_step(glass: &ShellGlass, distance: f32) -> f32 {
    let core = gaussian_step(glass.sigma, distance);
    if glass.halo_weight == 0.0 {
        return core;
    }
    (1.0 - glass.halo_weight) * core + glass.halo_weight * gaussian_step(glass.halo_sigma, distance)
}

fn tap(texture: &[f32], at: f32) -> f32 {
    let index = at - 0.5;
    let last = texture.len() - 1;
    if index <= 0.0 {
        return texture[0];
    }
    let low = index.floor() as usize;
    if low >= last {
        return texture[last];
    }
    texture[low] + (texture[low + 1] - texture[low]) * (index - low as f32)
}

fn kawase_down(texture: &[f32], offset: f32) -> Vec<f32> {
    let spread = 0.5 * offset;
    (0..texture.len() / 2)
        .map(|texel| {
            let centre = 2.0 * texel as f32 + 1.0;
            (2.0 * tap(texture, centre - spread)
                + 4.0 * tap(texture, centre)
                + 2.0 * tap(texture, centre + spread))
                / 8.0
        })
        .collect()
}

fn kawase_up(texture: &[f32], offset: f32) -> Vec<f32> {
    let spread = 0.5 * offset;
    (0..texture.len() * 2)
        .map(|texel| {
            let centre = (texel as f32 + 0.5) / 2.0;
            (tap(texture, centre - 2.0 * spread)
                + 4.0 * tap(texture, centre - spread)
                + 2.0 * tap(texture, centre)
                + 4.0 * tap(texture, centre + spread)
                + tap(texture, centre + 2.0 * spread))
                / 12.0
        })
        .collect()
}

pub fn kawase_sigma(passes: u32, offset: f32) -> f32 {
    let size = 4096;
    let mut texture: Vec<f32> = (0..size).map(|x| f32::from(x >= size / 2)).collect();
    for _ in 0..passes {
        texture = kawase_down(&texture, offset);
    }
    for _ in 0..passes {
        texture = kawase_up(&texture, offset);
    }
    let weights: Vec<f32> = texture.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let total: f32 = weights.iter().sum();
    let at = |index: usize| index as f32 + 0.5;
    let mean: f32 = weights
        .iter()
        .enumerate()
        .map(|(index, weight)| at(index) * weight / total)
        .sum();
    weights
        .iter()
        .enumerate()
        .map(|(index, weight)| (at(index) - mean).powi(2) * weight / total)
        .sum::<f32>()
        .sqrt()
}

pub fn kawase_reaching(sigma: f32, widest_offset: f32) -> (u32, f32) {
    for passes in 1.. {
        if kawase_sigma(passes, widest_offset) < sigma {
            continue;
        }
        let (mut low, mut high) = (0.0, widest_offset);
        for _ in 0..24 {
            let offset = 0.5 * (low + high);
            if kawase_sigma(passes, offset) < sigma {
                low = offset;
            } else {
                high = offset;
            }
        }
        return (passes, 0.5 * (low + high));
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use ctdesign::{
        SHELL_GLASS_CONTROL_CENTER_DARK, SHELL_GLASS_CONTROL_CENTER_PANEL_DARK,
        SHELL_GLASS_DOCK_DARK, SHELL_GLASS_MENU_DARK,
    };

    use super::*;

    const SAMPLES: &str = include_str!("../tests/data/shell-glass-samples.csv");

    fn glass(token: &str) -> ShellGlass {
        match token {
            "menu" => SHELL_GLASS_MENU_DARK,
            "control-center" => SHELL_GLASS_CONTROL_CENTER_DARK,
            "control-center-panel" => SHELL_GLASS_CONTROL_CENTER_PANEL_DARK,
            "dock" => SHELL_GLASS_DOCK_DARK,
            other => panic!("no glass `{other}`"),
        }
    }

    fn device(levels: &[u8]) -> Rgba {
        Rgba::rgb8(levels[0], levels[1], levels[2])
    }

    #[test]
    fn every_measured_gray_and_colour_comes_out_within_two_levels_unless_a_channel_clips() {
        let mut checked = 0;
        for line in SAMPLES.lines() {
            let mut fields = line.split(',');
            let token = fields.next().unwrap();
            let numbers: Vec<u8> = fields.skip(1).map(|field| field.parse().unwrap()).collect();
            let shown = over(&glass(token), device(&numbers[..3]));
            for (channel, shown) in [shown.r, shown.g, shown.b].into_iter().enumerate() {
                let measured = numbers[3 + channel];
                if (3..=252).contains(&measured) {
                    let error = (shown * 255.0 - f32::from(measured)).abs();
                    assert!(
                        error <= 2.0,
                        "{line}: channel {channel} is {}",
                        shown * 255.0
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 400, "only {checked} channels checked");
    }

    #[test]
    fn the_dual_kawase_of_the_wallpaper_blur_spreads_like_a_gaussian_of_17_pixels() {
        assert!((kawase_sigma(3, 3.0) - 17.16).abs() < 0.1);
        assert!((kawase_sigma(5, 1.0) - 25.0).abs() < 0.2);
    }

    #[test]
    fn the_passes_and_offset_that_reach_a_sigma_give_that_sigma() {
        for sigma in [4.0, 8.5, 17.16, 30.0] {
            let (passes, offset) = kawase_reaching(sigma, 3.0);
            assert!(
                (kawase_sigma(passes, offset) - sigma).abs() < 0.05,
                "sigma {sigma}"
            );
            assert!(offset <= 3.0);
        }
        assert_eq!(kawase_reaching(17.16, 3.0).0, 3);
    }

    #[test]
    fn the_step_of_a_gaussian_is_half_way_at_the_edge_and_84_percent_one_sigma_after() {
        let glass = ShellGlass {
            halo_weight: 0.0,
            ..glass("menu")
        };
        assert!((blurred_step(&glass, 0.0) - 0.5).abs() < 1e-6);
        assert!((blurred_step(&glass, glass.sigma) - 0.8413).abs() < 1e-3);
    }
}
