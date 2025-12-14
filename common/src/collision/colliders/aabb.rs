use nalgebra::{Point3, Vector3};

use crate::{collision::collidable::Intersectable, maths::Ray};

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
    fn intersect_t(&self, ray: &Ray) -> Option<f32> {
        let mut t1 = (self.min[0] - ray.origin[0]) * ray.direction_inv()[0];
        let mut t2 = (self.max[0] - ray.origin[0]) * ray.direction_inv()[0];

        let mut tmin = t1.min(t2);
        let mut tmax = t1.max(t2);

        for i in 1..3 {
            t1 = (self.min[i] - ray.origin[i]) * ray.direction_inv()[i];
            t2 = (self.max[i] - ray.origin[i]) * ray.direction_inv()[i];

            tmin = tmin.max(t1.min(t2).min(tmax));
            tmax = tmax.min(t1.max(t2).max(tmin));
        }

        if tmax > tmin.max(0.0) { Some(tmin) } else { None }
    }
}
