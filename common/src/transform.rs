use nalgebra::{Similarity3, Translation3, UnitQuaternion};
use serde::{Deserialize, Serialize};

use crate::maths;

#[derive(Clone, Serialize, Deserialize, Debug)]
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

    pub fn combine(&self, parent: &Transform) -> Transform {
        let mut combined = self.clone();

        // log::debug!("From {:?}", combined);

        combined.scale *= parent.scale;
        combined.rotation = parent.rotation * combined.rotation;
        combined.translation = Translation3::from(
            parent.translation.vector
                + parent
                    .rotation
                    .transform_vector(&(combined.translation.vector * parent.scale)),
        );

        // log::debug!("To {:?}", combined);

        combined.dirty = true;
        combined.compute_transform_matrix();
        combined
    }

    pub fn matrix(&self) -> [[f32; 4]; 4] {
        #[cfg(debug_assertions)]
        if self.dirty {
            log::warn!("Obtaining dirty transform matrix.")
        }

        self.matrix
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.dirty = true;
    }

    pub fn get_rotation(&self) -> UnitQuaternion<f32> {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: UnitQuaternion<f32>) {
        self.rotation = rotation;
        self.dirty = true;
    }

    pub fn get_translation(&self) -> Translation3<f32> {
        self.translation
    }

    pub fn set_translation(&mut self, translation: Translation3<f32>) {
        self.translation = translation;
        self.dirty = true;
    }

    pub fn identity() -> Self {
        Self::default()
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
