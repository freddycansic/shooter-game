use nalgebra::{Matrix4, Scale3, Similarity3, Transform3, Translation3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

use crate::maths;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transform {
    translation: Translation3<f32>,
    rotation: UnitQuaternion<f32>,
    scale: Scale3<f32>,
    #[serde(skip)]
    matrix: Matrix4<f32>,
    #[serde(skip)]
    dirty: bool,
}

impl Transform {
    pub fn compute_transform_matrix(&mut self) {
        if self.dirty {
            let scale_matrix = Matrix4::new_nonuniform_scaling(&self.scale.vector);
            let rotation_matrix = self.rotation.to_homogeneous();
            let translation_matrix = Translation3::from(self.translation).to_homogeneous();

            self.matrix = translation_matrix * rotation_matrix * scale_matrix;

            self.dirty = false;
        }
    }

    pub fn combine(&self, parent: &Transform) -> Transform {
        let mut combined = self.clone();

        combined.scale.x *= parent.scale.x;
        combined.scale.y *= parent.scale.y;
        combined.scale.z *= parent.scale.z;

        combined.rotation = parent.rotation * combined.rotation;

        let scaled_translation = Vector3::new(
            combined.translation.vector.x * parent.scale.x,
            combined.translation.vector.y * parent.scale.y,
            combined.translation.vector.z * parent.scale.z,
        );

        combined.translation = Translation3::from(
            parent.translation.vector + parent.rotation.transform_vector(&scaled_translation)
        );

        combined.dirty = true;
        combined.compute_transform_matrix();
        combined
    }

    pub fn raw_matrix(&self) -> [[f32; 4]; 4] {
        maths::raw_matrix(self.matrix().into())
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        #[cfg(debug_assertions)]
        if self.dirty {
            log::warn!("Obtaining dirty transform matrix.")
        }

        self.matrix
    }

    pub fn scale(&self) -> Scale3<f32> {
        self.scale
    }

    pub fn set_scale(&mut self, scale: Scale3<f32>) {
        self.scale = scale;
        self.dirty = true;
    }

    pub fn rotation(&self) -> UnitQuaternion<f32> {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: UnitQuaternion<f32>) {
        self.rotation = rotation;
        self.dirty = true;
    }

    pub fn translation(&self) -> Translation3<f32> {
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
            scale: Scale3::identity(),
            matrix: Matrix4::identity(),
            dirty: false,
        }
    }
}
