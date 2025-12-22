use nalgebra::Point3;

use crate::{
    collision::collidable::{Hit, Intersectable},
    maths::Ray,
};

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
}

impl Intersectable for Aabb {
    fn intersect_t(&self, ray: &Ray) -> Option<Hit> {
        let mut tmin = f32::NEG_INFINITY; // earliest possible intersection
        let mut tmax = f32::INFINITY; // lastest possible intersection

        for i in 0..3 {
            if ray.direction()[i] != 0.0 {
                let t1 = (self.min[i] - ray.origin[i]) * ray.direction_inv()[i];
                let t2 = (self.max[i] - ray.origin[i]) * ray.direction_inv()[i];

                tmin = tmin.max(t1.min(t2));
                tmax = tmax.min(t1.max(t2));
            } else if ray.origin[i] < self.min[i] || ray.origin[i] > self.max[i] {
                return None;
            }
        }

        if tmax >= tmin && tmax > 0.0 {
            Some(Hit {
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
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn intersect_t_aabb_corner_hit() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0).normalize());

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 3_f32.sqrt());
    }

    #[test]
    fn intersect_t_aabb_face_hit() {
        let ray = Ray::new(Point3::new(0.0, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
    }

    #[test]
    fn intersect_t_aabb_edge_hit() {
        let ray = Ray::new(Point3::new(0.0, 1.0, 1.0), Vector3::new(1.0, 0.0, 0.0));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
    }

    #[test]
    fn intersect_t_ray_inside_aabb() {
        let ray = Ray::new(Point3::new(1.5, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.intersect_t(&ray).unwrap();

        assert_relative_eq!(result.tmin, 0.0);
        assert_relative_eq!(result.tmax, 0.5);
    }

    #[test]
    fn intersect_t_aabb_miss_parallel() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        assert!(aabb.intersect_t(&ray).is_none());
    }

    #[test]
    fn intersect_t_aabb_behind_ray() {
        let ray = Ray::new(Point3::new(3.0, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        assert!(aabb.intersect_t(&ray).is_none());
    }

    #[test]
    fn intersect_t_aabb_grazing_hit() {
        let ray = Ray::new(Point3::new(0.0, 2.0, 1.5), Vector3::new(1.0, 0.0, 0.0));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
    }
}
