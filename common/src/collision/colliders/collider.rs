use crate::{
    collision::{
        collidable::{Hit, Intersectable},
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
    pub fn intersection(&self, ray: &Ray) -> Option<Hit> {
        match self {
            Collider::Aabb(aabb) => aabb.intersect_t(ray),
            Collider::Bvh(bvh) => bvh.intersect_t(ray),
        }
    }
}
