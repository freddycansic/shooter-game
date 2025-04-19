use rapier3d::na::{Matrix4, Similarity3, Translation3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

use crate::maths;

#[derive(Clone, Serialize, Deserialize)]
pub struct Transform {
    translation: Translation3<f32>,
    rotation: UnitQuaternion<f32>,
    scale: f32,
    matrix: [[f32; 4]; 4],
    dirty: bool,
}

impl Transform {
    pub fn compute_transform_matrix(&mut self) {
        if self.dirty {
            let mut matrix = Similarity3::identity();
            matrix.append_translation_mut(&self.translation);
            matrix.append_rotation_mut(&self.rotation);
            matrix.append_scaling_mut(self.scale);

            self.matrix = maths::raw_matrix(matrix.into());

            self.dirty = false;
        }
    }

    pub fn matrix(&self) -> [[f32; 4]; 4] {
        #[cfg(debug_assertions)]
        if self.dirty {
            log::warn!("Obtaining dirty transform matrix.")
        }

        self.matrix
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.dirty = true;
    }

    pub fn set_rotation(&mut self, rotation: UnitQuaternion<f32>) {
        self.rotation = rotation;
        self.dirty = true;
    }

    pub fn set_translation(&mut self, translation: Translation3<f32>) {
        self.translation = translation;
        self.dirty = true;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Translation3::identity(),
            rotation: UnitQuaternion::identity(),
            scale: 1.0,
            matrix: maths::raw_identity_matrix(),
            dirty: false,
        }
    }
}
