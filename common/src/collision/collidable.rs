use nalgebra::Vector3;

use crate::{collision::colliders::Collider, maths::Ray};

pub trait Collidable {
    fn collider<'a>(&self) -> &'a Collider;
}

pub struct Hit {
    pub tmin: f64, // entry point
    pub tmax: f64, // exit point
}

pub trait Intersectable {
    fn intersect_t(&self, ray: &Ray) -> Option<Hit>;
}
