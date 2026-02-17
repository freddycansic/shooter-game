use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use crate::collision::collidable::{BroadPhaseCollisionQuery, NarrowPhaseCollisionQuery, Sweep};
use crate::collision::colliders::capsule::Capsule;
use crate::collision::colliders::sphere::Sphere;
use crate::maths::Local;
use crate::engine::resources::Resources;
use crate::{collision::collidable::RayHit, maths::Ray};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Aabb {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl Aabb {
    pub fn union(&self, other: &Self) -> Self {
        Aabb {
            min: Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    fn intersects_capsule_t(&self, t: f32, capsule: &Capsule, ba: &Vector3<f32>) -> bool {
        let p_t = capsule.p1 + t * ba;

        let pb_t = Point3::new(
            p_t.x.clamp(self.min.x, self.max.x),
            p_t.y.clamp(self.min.y, self.max.y),
            p_t.z.clamp(self.min.z, self.max.z),
        );

        (pb_t - p_t).magnitude_squared() <= capsule.radius * capsule.radius
    }
}

impl BroadPhaseCollisionQuery<Local<Sphere>> for Aabb {
    fn broad_intersect(&self, sphere: &Local<Sphere>, _resources: &Resources) -> bool {
        let clamped = Point3::new(
            sphere.origin.x.clamp(self.min.x, self.max.x),
            sphere.origin.y.clamp(self.min.y, self.max.y),
            sphere.origin.z.clamp(self.min.z, self.max.z),
        );

        (clamped - sphere.origin).magnitude_squared() <= sphere.radius * sphere.radius
    }
}

impl BroadPhaseCollisionQuery<Local<Sweep<Sphere>>> for Aabb {
    fn broad_intersect(&self, query: &Local<Sweep<Sphere>>, resources: &Resources) -> bool {
        let swept_sphere = Local(Capsule::new(
            query.object.origin,
            query.object.origin + query.velocity,
            query.object.radius,
        ));

        self.broad_intersect(&swept_sphere, resources)
    }
}

impl BroadPhaseCollisionQuery<Local<Capsule>> for Aabb {
    fn broad_intersect(&self, capsule: &Local<Capsule>, _resources: &Resources) -> bool {
        let ba = capsule.p2 - capsule.p1;

        // Test endpoints
        for end in [0.0, 1.0] {
            if self.intersects_capsule_t(end, capsule, &ba) {
                return true;
            }
        }

        // Test min max of each slab
        for i in 0..=2 {
            for point in [self.min, self.max] {
                if ba[i] == 0.0 {
                    continue;
                }

                let t = (point[i] - capsule.p1[i]) / ba[i];

                // Only test values within the line segment
                if t < 0.0 || t > 1.0 {
                    continue;
                }

                if self.intersects_capsule_t(t, capsule, &ba) {
                    return true;
                }
            }
        }

        false
    }
}

impl NarrowPhaseCollisionQuery<Local<Ray>> for Aabb {
    type Hit = Option<RayHit>;

    fn narrow_intersect(&self, local_ray: &Local<Ray>, _resources: &Resources) -> Option<RayHit> {
        let mut tmin = f32::NEG_INFINITY; // earliest possible intersection
        let mut tmax = f32::INFINITY; // latest possible intersection

        for i in 0..3 {
            if local_ray.direction()[i] != 0.0 {
                let t1 = (self.min[i] - local_ray.origin[i]) * local_ray.direction_inv()[i];
                let t2 = (self.max[i] - local_ray.origin[i]) * local_ray.direction_inv()[i];

                tmin = tmin.max(t1.min(t2));
                tmax = tmax.min(t1.max(t2));
            } else if local_ray.origin[i] < self.min[i] || local_ray.origin[i] > self.max[i] {
                return None;
            }
        }

        if tmax >= tmin && tmax > 0.0 {
            Some(RayHit {
                tmin: tmin.max(0.0),
                tmax,
            })
        } else {
            None
        }
    }
}
