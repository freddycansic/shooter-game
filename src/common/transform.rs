use cgmath::{Matrix4, Quaternion, Vector3, Zero};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: f32,
}

impl From<Transform> for Matrix4<f32> {
    fn from(value: Transform) -> Self {
        Matrix4::from_translation(value.translation)
            * Matrix4::from(value.rotation)
            * Matrix4::from_scale(value.scale)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: 1.0,
        }
    }
}
