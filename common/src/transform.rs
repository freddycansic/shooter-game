use rapier3d::na::{Matrix4, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: f32,
}

impl From<Transform> for Matrix4<f32> {
    fn from(value: Transform) -> Self {
        Matrix4::new_translation(&value.translation)
            * value.rotation.to_homogeneous()
            * Matrix4::new_scaling(value.scale)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3::zeros(),
            rotation: UnitQuaternion::identity(),
            scale: 1.0,
        }
    }
}
