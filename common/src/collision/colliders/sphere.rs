use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Sphere {
    pub radius: f32,
    pub origin: Point3<f32>,
}

impl Sphere {
    pub fn new(origin: Point3<f32>, radius: f32) -> Sphere {
        Sphere { origin, radius }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0)
    }
}
