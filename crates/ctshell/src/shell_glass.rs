use ctdesign::{Rgba, SHELL_GLASS_CHROMA_KNOTS, SHELL_GLASS_LEVELS, ShellGlass};

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
    let luma = blurred.luma();
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
