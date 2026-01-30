use std::cmp::Ordering;
use crate::collision::colliders::capsule::Capsule;
use crate::collision::colliders::sphere::Sphere;
use crate::maths::Ray;
use nalgebra::Vector3;

#[derive(PartialEq)]
pub struct RayHit {
    pub tmin: f32, // entry point
    pub tmax: f32, // exit point
}

impl PartialOrd for RayHit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.tmin.partial_cmp(&other.tmin)
    }
}

#[derive(PartialEq, Debug)]
pub struct SweepHit {
    pub t: f32, // time of hit along velocity
    pub normal: Vector3<f32>,
    pub point: Vector3<f32>,
}

impl PartialOrd for SweepHit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.t.partial_cmp(&other.t)
    }
}

pub trait Intersectable {
    fn intersect_ray(&self, _ray: &Ray) -> Option<RayHit> {
        unimplemented!("Ray intersection unsupported.");
    }

    fn intersects_capsule(&self, _capsule: &Capsule) -> bool {
        unimplemented!("Capsule intersection unsupported.")
    }
    fn intersects_sphere(&self, _sphere: &Sphere) -> bool {
        unimplemented!("Sphere intersection unsupported.")
    }

    fn sweep_intersects_sphere(&self, _sphere: &Sphere, _velocity: &Vector3<f32>) -> bool {
        unimplemented!("Sweep sphere intersection unsupported.")
    }
    fn sweep_intersect_sphere(&self, _sphere: &Sphere, _velocity: &Vector3<f32>) -> Option<SweepHit> {
        unimplemented!("Sweep sphere intersection unsupported.")
    }
}