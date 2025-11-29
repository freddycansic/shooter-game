use nalgebra::Matrix4;

pub fn linear_map(
    x: f32,
    original_min: f32,
    original_max: f32,
    target_min: f32,
    target_max: f32,
) -> f32 {
    ((x - original_min) * (target_max - target_min) / (original_max - original_min)) + target_min
}

pub fn raw_matrix(matrix: Matrix4<f32>) -> [[f32; 4]; 4] {
    <[[f32; 4]; 4]>::from(matrix)
}

pub fn raw_identity_matrix() -> [[f32; 4]; 4] {
    raw_matrix(Matrix4::identity())
}

pub fn perspective_matrix_from_dimensions(width: f32, height: f32) -> Matrix4<f32> {
    const MIN_ASPECT_RATIO: f32 = 0.1;
    let aspect_ratio = (width / height).max(MIN_ASPECT_RATIO);

    Matrix4::new_perspective(aspect_ratio, std::f32::consts::FRAC_PI_2, 0.01, 2000.0)
}

pub fn orthographic_matrix_from_dimensions(width: f32, height: f32) -> Matrix4<f32> {
    const MIN_DIMENSION: f32 = 1.0;
    let safe_width = width.max(MIN_DIMENSION);
    let safe_height = height.max(MIN_DIMENSION);

    Matrix4::new_orthographic(0.0, safe_width, 0.0, safe_height, 0.01, 100.0)
}

pub trait Matrix4Ext {
    // This function removes the w components of a 4x4 matrix, and sets the diagonal on the 4th row to 1
    fn stripped_w(self) -> Self;
}

impl Matrix4Ext for Matrix4<f32> {
    fn stripped_w(self) -> Self {
        let mut stripped = self.fixed_resize::<3, 3>(0.0).fixed_resize::<4, 4>(0.0);

        stripped[(3, 3)] = 1.0;

        stripped
    }
}
