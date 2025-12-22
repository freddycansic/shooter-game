use nalgebra::{Point3, Vector3};

pub struct Ray {
    pub origin: Point3<f64>,

    direction: Vector3<f64>,
    direction_inv: Vector3<f64>,
}

impl Ray {
    pub fn new(origin: Point3<f64>, direction: Vector3<f64>) -> Self {
        Self {
            origin,
            direction,
            direction_inv: Vector3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
        }
    }

    pub fn direction(&self) -> Vector3<f64> {
        self.direction
    }

    pub fn direction_inv(&self) -> Vector3<f64> {
        self.direction_inv
    }
}
