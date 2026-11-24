const KERNEL_REACH: f32 = 3.5;

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
