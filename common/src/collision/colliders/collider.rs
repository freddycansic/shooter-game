use nalgebra::Vector3;

use crate::{
    collision::{
        collidable::Intersectable,
        colliders::{aabb::Aabb, bvh::Bvh},
    },
    maths::Ray,
};

pub enum Collider {
    Aabb(Aabb),
    Bvh(Bvh),
    // Sphere(Sphere),
    // Capsule(Capsule),
}

impl Collider {
    pub fn intersection(&self, ray: &Ray) -> Option<Vector3<f32>> {
        match self {
            Collider::Aabb(aabb) => aabb.intersect_t(ray),
            Collider::Bvh(bvh) => bvh.intersect_t(ray),
        }
    }
}
