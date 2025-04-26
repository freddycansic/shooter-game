use rapier3d::na::Matrix4;

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

pub fn perspective_matrix_from_window_size(window_width: f32, window_height: f32) -> Matrix4<f32> {
    Matrix4::new_perspective(
        window_width / window_height,
        std::f32::consts::FRAC_PI_2,
        0.01,
        2000.0,
    )
}

pub fn orthographic_matrix_from_window_size(window_width: f32, window_height: f32) -> Matrix4<f32> {
    Matrix4::new_orthographic(0.0, window_width, 0.0, window_height, 0.01, 100.0)
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
