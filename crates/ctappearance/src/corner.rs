use ctdesign::CORNER_CONCENTRIC_REGION;

pub fn concentric(container: f32, inset_x: f32, inset_y: f32, minimum: f32) -> f32 {
    let limit = container * CORNER_CONCENTRIC_REGION;
    if inset_x > limit || inset_y > limit {
        return minimum;
    }
    (container - inset_x.min(inset_y)).max(minimum)
}
