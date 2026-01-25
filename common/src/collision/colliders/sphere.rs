use nalgebra::Vector3;
use crate::collision::collidable::Intersectable;

pub struct Sphere {
    pub radius: f32,
    pub origin: Vector3<f32>
}

impl Sphere {
    pub fn new(origin: Vector3<f32>, radius: f32) -> Sphere {
        Sphere { origin, radius }
    }
}