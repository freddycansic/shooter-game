use cgmath::{Matrix4, Quaternion, Vector3, Zero};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl From<Transform> for Matrix4<f32> {
    fn from(value: Transform) -> Self {
        Matrix4::from_translation(value.translation)
            * Matrix4::from(value.rotation)
            * Matrix4::from_nonuniform_scale(value.scale.x, value.scale.y, value.scale.z)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
