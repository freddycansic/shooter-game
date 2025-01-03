use cgmath::{Matrix3, Matrix4};

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

pub trait Matrix4Ext {
    fn to_matrix3(self) -> Matrix3<f32>;
}

impl Matrix4Ext for Matrix4<f32> {
    fn to_matrix3(self) -> Matrix3<f32> {
        Matrix3::from_cols(self.x.xyz(), self.y.xyz(), self.z.xyz())
    }
}
