use crate::collision::collidable::{NarrowPhaseCollisionQuery, RayHit};
use crate::collision::colliders::cylinder;
use crate::maths::{Local, Ray};
use crate::engine::resources::Resources;
use nalgebra::Point3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Capsule {
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub radius: f32,
}

impl Capsule {
    pub fn new(p1: Point3<f32>, p2: Point3<f32>, radius: f32) -> Self {
        Self { p1, p2, radius }
    }
}

impl NarrowPhaseCollisionQuery<Local<Ray>> for Capsule {
    type Hit = Option<RayHit>;

    fn narrow_intersect(&self, ray: &Local<Ray>, _resources: &Resources) -> Option<RayHit> {
        let length_squared = (self.p1 - self.p2).magnitude_squared();

        // it's just a sphere
        if length_squared == 0.0 {
            return ray_sphere(ray, &self.p1, self.radius);
        }

        let mut tmin = f32::INFINITY;
        let mut tmax = f32::NEG_INFINITY;

        if let Some(hit) = cylinder::intersect(ray, &self.p1, &self.p2, self.radius) {
            tmin = hit.tmin;
            tmax = hit.tmax;
        }

        // end spheres
        for end in &[self.p1, self.p2] {
            if let Some(hit) = ray_sphere(ray, end, self.radius) {
                tmin = tmin.min(hit.tmin);
                tmax = tmax.max(hit.tmax);
            }
        }

        (tmin <= tmax).then(|| RayHit { tmin, tmax })
    }
}

pub fn ray_sphere(ray: &Ray, center: &Point3<f32>, radius: f32) -> Option<RayHit> {
    let oc = (ray.origin - center).to_homogeneous().xyz();
    let a = ray.direction().dot(&ray.direction());
    let b = 2.0 * oc.dot(&ray.direction());
    let c = oc.dot(&oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None; // no intersection
    }

    let sqrt_disc = discriminant.sqrt();
    let t0 = (-b - sqrt_disc) / (2.0 * a);
    let t1 = (-b + sqrt_disc) / (2.0 * a);

    // Ensure entry <= exit
    let tmin = t0.min(t1);
    let tmax = t0.max(t1);

    // Only consider intersections in front of the ray
    if tmax < 0.0 {
        None
    } else {
        Some(RayHit {
            tmin: tmin.max(0.0),
            tmax,
        })
    }
}
