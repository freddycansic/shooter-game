pub fn linear_map(
    x: f32,
    original_min: f32,
    original_max: f32,
    target_min: f32,
    target_max: f32,
) -> f32 {
    ((x - original_min) * (target_max - target_min) / (original_max - original_min)) + target_min
}
