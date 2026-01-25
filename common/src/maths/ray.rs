use approx::RelativeEq;
use nalgebra::{Point3, Vector3};

pub struct Ray {
    pub origin: Vector3<f32>,

    direction: Vector3<f32>,
    direction_inv: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Self {
        debug_assert!(
            direction
                .normalize()
                .relative_eq(&direction, f32::EPSILON, f32::EPSILON),
            "The direction is not normalised"
        );

        Self {
            origin,
            direction,
            direction_inv: Vector3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
        }
    }

    pub fn direction(&self) -> Vector3<f32> {
        self.direction
    }

    pub fn direction_inv(&self) -> Vector3<f32> {
        self.direction_inv
    }
}
