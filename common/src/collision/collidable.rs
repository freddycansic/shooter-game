use crate::collision::colliders::capsule::Capsule;
use crate::collision::colliders::sphere::Sphere;
use crate::maths::Ray;
use crate::resources::Resources;
use nalgebra::{Point3, Vector3};
use petgraph::prelude::NodeIndex;

#[derive(Debug)]
pub struct RayHit {
    pub tmin: f32, // entry point
    pub tmax: f32, // exit point
}

#[derive(Debug)]
pub struct RayHitNode {
    pub hit: RayHit,
    pub node: NodeIndex,
}

pub struct Sweep<T> {
    pub object: T,
    pub velocity: Vector3<f32>,
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

pub trait BroadPhaseCollisionQuery<T> {
    fn broad_intersect(&self, query: &T, resources: &Resources) -> bool;
}

pub trait NarrowPhaseCollisionQuery<T> {
    type Hit;

    fn narrow_intersect(&self, query: &T, resources: &Resources) -> Self::Hit;
}
