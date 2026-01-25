use nalgebra::Vector3;
use crate::{maths::Ray};
use crate::collision::colliders::capsule::Capsule;
use crate::collision::colliders::sphere::Sphere;

pub struct RayHit {
    pub tmin: f32, // entry point
    pub tmax: f32, // exit point
}

pub struct SweepHit {
    pub t: f32, // time of hit along velocity
    pub normal: Vector3<f32>,
    pub point: Vector3<f32>,
}

pub trait Intersectable {
    fn intersect_ray(&self, ray: &Ray) -> Option<RayHit> { unimplemented!("Ray intersection unsupported.");}

    fn intersects_capsule(&self, capsule: &Capsule) -> bool { unimplemented!("Capsule intersection unsupported.") }
    fn intersects_sphere(&self, sphere: &Sphere) -> bool { unimplemented!("Sphere intersection unsupported.") }

    fn sweep_intersects_sphere(&self, sphere: &Sphere, velocity: &Vector3<f32>) -> bool { unimplemented!("Sweep sphere intersection unsupported.") }
    fn sweep_intersect_sphere(&self, sphere: &Sphere, velocity: &Vector3<f32>) -> Option<SweepHit> { unimplemented!("Sweep sphere intersection unsupported.") }
}
