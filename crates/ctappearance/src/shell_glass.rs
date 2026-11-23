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

#[cfg(test)]
mod tests {
    use ctdesign::{
        SHELL_GLASS_APPS_DARK, SHELL_GLASS_CONTROL_CENTER_DARK,
        SHELL_GLASS_CONTROL_CENTER_PANEL_DARK, SHELL_GLASS_DOCK_DARK, SHELL_GLASS_MENU_DARK,
        SHELL_GLASS_NOTIFICATION_CARD_DARK, SHELL_GLASS_SPACES_BAR_DARK,
        SHELL_GLASS_SPOTLIGHT_DARK,
    };

    use super::*;

    const SAMPLES: &str = include_str!("../tests/data/shell-glass-samples.csv");

    fn glass(token: &str) -> ShellGlass {
        match token {
            "menu" => SHELL_GLASS_MENU_DARK,
            "spotlight" => SHELL_GLASS_SPOTLIGHT_DARK,
            "apps" => SHELL_GLASS_APPS_DARK,
            "notification-card" => SHELL_GLASS_NOTIFICATION_CARD_DARK,
            "control-center" => SHELL_GLASS_CONTROL_CENTER_DARK,
            "control-center-panel" => SHELL_GLASS_CONTROL_CENTER_PANEL_DARK,
            "dock" => SHELL_GLASS_DOCK_DARK,
            other => panic!("no glass `{other}`"),
        }
    }

    fn rows() -> impl Iterator<Item = (&'static str, Vec<u8>)> {
        SAMPLES.lines().map(|line| {
            let mut fields = line.split(',');
            let token = fields.next().unwrap();
            (
                token,
                fields.skip(1).map(|field| field.parse().unwrap()).collect(),
            )
        })
    }

    fn within(shown: [f32; 3], measured: &[u8], tolerance: f32, line: &str) -> usize {
        let mut checked = 0;
        for (channel, shown) in shown.into_iter().enumerate() {
            let measured = measured[channel];
            if (3..=252).contains(&measured) {
                let error = (shown * 255.0 - f32::from(measured)).abs();
                assert!(
                    error <= tolerance,
                    "{line}: channel {channel} is {}",
                    shown * 255.0
                );
                checked += 1;
            }
        }
        checked
    }

    #[test]
    fn every_measured_gray_and_colour_comes_out_within_two_levels_unless_a_channel_clips() {
        let mut checked = 0;
        for (token, numbers) in rows().filter(|(token, _)| *token != "spaces-bar") {
            let shown = over(
                &glass(token),
                Rgba::rgb8(numbers[0], numbers[1], numbers[2]),
            );
            checked += within([shown.r, shown.g, shown.b], &numbers[3..], 2.0, token);
        }
        assert!(checked > 700, "only {checked} channels checked");
    }

    #[test]
    fn the_spaces_bar_is_a_saturation_a_veil_and_a_per_channel_cap_within_one_level() {
        let mut checked = 0;
        for (_, numbers) in rows().filter(|(token, _)| *token == "spaces-bar") {
            let backdrop =
                [numbers[0], numbers[1], numbers[2]].map(|level| f32::from(level) / 255.0);
            let shown = crate::material::over(&SHELL_GLASS_SPACES_BAR_DARK, backdrop);
            checked += within(shown, &numbers[3..], 1.0, "spaces-bar");
        }
        assert!(checked > 40, "only {checked} channels checked");
    }

    #[test]
    fn the_notification_card_and_the_menu_are_not_the_same_glass_and_the_others_are() {
        let tone_at = |token: &str| tone(&glass(token), 0.5);
        assert!((tone_at("menu") - tone_at("spotlight")).abs() < 0.004);
        assert!((tone_at("menu") - tone_at("apps")).abs() < 0.004);
        assert!(tone_at("menu") - tone_at("notification-card") > 0.05);
    }

    #[test]
    fn the_dual_kawase_of_the_wallpaper_blur_spreads_like_a_gaussian_of_17_pixels() {
        assert!((kawase_sigma(3, 3.0) - 17.16).abs() < 0.1);
        assert!((kawase_sigma(5, 1.0) - 25.0).abs() < 0.2);
    }
}
