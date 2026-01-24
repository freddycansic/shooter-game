use crate::{maths::Ray};
use crate::collision::colliders::capsule::Capsule;

pub struct RayHit {
    pub tmin: f32, // entry point
    pub tmax: f32, // exit point
}

pub trait Intersectable {
    fn intersect_ray(&self, ray: &Ray) -> Option<RayHit> { unimplemented!("Ray intersection unsupported.");}

    fn intersects_capsule(&self, capsule: &Capsule) -> bool { unimplemented!("Capsule intersection unsupported.") }
}
