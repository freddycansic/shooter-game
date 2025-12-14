use nalgebra::Vector3;

use crate::{collision::colliders::Collider, maths::Ray};

pub trait Collidable {
    fn collider<'a>(&self) -> &'a Collider;
}

pub trait Intersectable {
    fn intersect_t(&self, ray: &Ray) -> Option<f32>;
}
