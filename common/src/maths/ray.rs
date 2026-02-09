use crate::maths::Transform;
use crate::maths::local::Local;
use nalgebra::{Point3, Vector3};

pub struct Ray {
    pub origin: Point3<f32>,

    direction: Vector3<f32>,
    direction_inv: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Self {
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

    pub fn point_at(&self, t: f32) -> Point3<f32> {
        self.origin + self.direction * t
    }

    pub fn to_local(&self, transform: &Transform) -> Local<Ray> {
        let world_inverse = transform.matrix().try_inverse().unwrap();

        let local_origin = world_inverse.transform_point(&self.origin);
        let local_direction = world_inverse.transform_vector(&self.direction()).normalize();

        Local(Ray::new(local_origin, local_direction))
    }
}
