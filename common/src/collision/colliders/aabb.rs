use nalgebra::{Point3, Vector3};

use crate::collision::collidable::{BroadPhaseCollisionQuery, NarrowPhaseCollisionQuery, Sweep};
use crate::collision::colliders::capsule::Capsule;
use crate::collision::colliders::sphere::Sphere;
use crate::maths::Local;
use crate::resources::Resources;
use crate::{
    collision::collidable::{RayHit},
    maths::Ray,
};

#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nalgebra::Vector3;

    #[test]
    fn intersect_aabb_corner_hit() {
        let ray = Local(Ray::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0).normalize(),
        ));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 3_f32.sqrt());
    }

    #[test]
    fn intersect_aabb_face_hit() {
        let ray = Local(Ray::new(Point3::new(0.0, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
    }

    #[test]
    fn intersect_aabb_edge_hit() {
        let ray = Local(Ray::new(Point3::new(0.0, 1.0, 1.0), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
    }

    #[test]
    fn intersect_ray_inside_aabb() {
        let ray = Local(Ray::new(Point3::new(1.5, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Resources::new()).unwrap();

        assert_relative_eq!(result.tmin, 0.0);
        assert_relative_eq!(result.tmax, 0.5);
    }

    #[test]
    fn intersect_aabb_miss_parallel() {
        let ray = Local(Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        assert!(aabb.narrow_intersect(&ray, &Resources::new()).is_none());
    }

    #[test]
    fn intersect_aabb_behind_ray() {
        let ray = Local(Ray::new(Point3::new(3.0, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        assert!(aabb.narrow_intersect(&ray, &Resources::new()).is_none());
    }

    #[test]
    fn intersect_aabb_grazing_hit() {
        let ray = Local(Ray::new(Point3::new(0.0, 2.0, 1.5), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
    }

    // -----------------------------

    #[test]
    fn intersect_capsule_aabb_face_hit_capsule_segment() {
        let capsule = Local(Capsule::new(
            Point3::new(0.0, 1.5, 0.0),
            Point3::new(0.0, -0.5, 0.0),
            1.0,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_aabb_face_graze_capsule_end() {
        let capsule = Local(Capsule::new(
            Point3::new(0.0, 2.5, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            1.0,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_aabb_face_barely_miss_capsule_end() {
        let capsule = Local(Capsule::new(
            Point3::new(0.0, 2.5, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            0.99,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), false);
    }

    #[test]
    fn intersect_capsule_aabb_face_intersect_capsule_end() {
        let capsule = Local(Capsule::new(
            Point3::new(0.0, 2.5, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            1.5,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_aabb_face_miss_capsule_end() {
        let capsule = Local(Capsule::new(
            Point3::new(0.0, 7.5, 0.0),
            Point3::new(0.0, 5.0, 0.0),
            1.5,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), false);
    }

    #[test]
    fn intersect_capsule_aabb_corner_intersect_capsule_segment() {
        let capsule = Local(Capsule::new(
            Point3::new(5.0, 5.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
            1.0,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_aabb_corner_graze_capsule_end() {
        let capsule = Local(Capsule::new(
            Point3::new(5.0, 5.0, 5.0),
            Point3::new(2.0, 2.0, 2.0),
            3.0_f32.sqrt(),
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_aabb_corner_miss_capsule() {
        let capsule = Local(Capsule::new(
            Point3::new(5.0, 5.0, 5.0),
            Point3::new(2.0, 2.0, 2.0),
            1.0,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), false);
    }

    #[test]
    fn intersect_capsule_aabb_edge_hit_capsule_segment() {
        let capsule = Local(Capsule::new(
            Point3::new(-2.0, -2.0, 0.0),
            Point3::new(-2.0, 2.0, 0.0),
            1.2,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_aabb_edge_graze_capsule_segment() {
        let capsule = Local(Capsule::new(
            Point3::new(-2.0, -2.0, 0.0),
            Point3::new(-2.0, 2.0, 0.0),
            1.0,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_aabb_edge_barely_miss_capsule() {
        let capsule = Local(Capsule::new(
            Point3::new(-2.0, -2.0, 0.0),
            Point3::new(-2.0, 2.0, 0.0),
            0.99,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), false);
    }

    #[test]
    fn intersect_capsule_capsule_inside_aabb() {
        let capsule = Local(Capsule::new(
            Point3::new(0.0, -0.1, 0.0),
            Point3::new(0.0, 0.1, 0.0),
            0.5,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_aabb_inside_capsule() {
        let capsule = Local(Capsule::new(
            Point3::new(0.0, -0.1, 0.0),
            Point3::new(0.0, 0.1, 0.0),
            50.0,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }

    #[test]
    fn intersect_capsule_capsule_length_zero() {
        let capsule = Local(Capsule::new(
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            1.0,
        ));

        let aabb = Aabb {
            min: Point3::new(-1.0, -1.0, -1.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };

        assert_eq!(aabb.broad_intersect(&capsule, &Resources::new()), true);
    }
}
