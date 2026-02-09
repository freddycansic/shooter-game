use crate::collision::colliders::capsule::Capsule;
use crate::collision::colliders::sphere::Sphere;
use crate::maths::Ray;
use nalgebra::{Point3, Vector3};
use petgraph::prelude::NodeIndex;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct RayHit {
    pub tmin: f32, // entry point
    pub tmax: f32, // exit point
}

pub struct RayHitNode {
    pub hit: RayHit,
    pub node: NodeIndex,
}

#[derive(Debug)]
pub struct SweepHit {
    pub t: f32, // time of hit along velocity
    pub normal: Vector3<f32>,
    pub point: Point3<f32>,
}

pub struct SweepHitNode {
    pub hit: SweepHit,
    pub node: NodeIndex,
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
