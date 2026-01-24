use nalgebra::Vector3;
use crate::collision::collidable::{Hit, Intersectable};
use crate::maths::Ray;

pub struct Capsule {
    pub p1: Vector3<f32>,
    pub p2: Vector3<f32>,
    pub radius: f32,
}

impl Intersectable for Capsule {
    fn intersect_t(&self, ray: &Ray) -> Option<Hit> {
        let line_segment = self.p2 - self.p1;
        let length = line_segment.norm();

        // it's just a sphere
        if length == 0.0 {
            return ray_sphere(ray, self.p1, self.radius);
        }

        let unit_segment = line_segment / length;
        let p1_to_ray_origin = (ray.origin - self.p1).to_homogeneous().xyz();
        let d_perp = ray.direction() - unit_segment * ray.direction().dot(&unit_segment);
        let ao_perp = p1_to_ray_origin - unit_segment * p1_to_ray_origin.dot(&unit_segment);

        let a_q = d_perp.dot(&d_perp);
        let b_q = 2.0 * d_perp.dot(&ao_perp);
        let c_q = ao_perp.dot(&ao_perp) - self.radius * self.radius;

        let mut tmin = f32::INFINITY;
        let mut tmax = f32::NEG_INFINITY;

        if a_q > 0.0 {
            let disc = b_q * b_q - 4.0 * a_q * c_q;
            if disc >= 0.0 {
                let sqrt_disc = disc.sqrt();
                for &t in &[(-b_q - sqrt_disc) / (2.0 * a_q), (-b_q + sqrt_disc) / (2.0 * a_q)] {
                    if t >= 0.0 {
                        // is the collision contained within the line segment?
                        let y = (p1_to_ray_origin + ray.direction() * t).dot(&unit_segment);
                        if y >= 0.0 && y <= length {
                            tmin = tmin.min(t);
                            tmax = tmax.max(t);
                        }
                    }
                }
            }
        }

        // end spheres
        for end in &[self.p1, self.p2] {
            if let Some(hit) = ray_sphere(ray, *end, self.radius) {
                tmin = tmin.min(hit.tmin);
                tmax = tmax.max(hit.tmax);
            }
        }

        if tmin <= tmax {
            Some(Hit { tmin, tmax })
        } else {
            None
        }
    }
}

fn ray_sphere(ray: &Ray, center: Vector3<f32>, radius: f32) -> Option<Hit> {
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
        Some(Hit {
            tmin: tmin.max(0.0),
            tmax,
        })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use nalgebra::Point3;
    use super::*;

    #[test]
    fn intersect_t_zero_length_origin_capsule_hit() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, 0.0, 0.0),
            p2: Vector3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        };

        let ray = Ray::new(Point3::new(-2.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));

        let result = capsule.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 3.0);
    }

    #[test]
    fn intersect_t_zero_length_origin_capsule_miss() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, 0.0, 0.0),
            p2: Vector3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        };

        let ray = Ray::new(Point3::new(-2.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));

        let result = capsule.intersect_t(&ray);
        assert!(result.is_none());
    }

    #[test]
    fn intersect_t_zero_length_origin_capsule_graze() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, 0.0, 0.0),
            p2: Vector3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        };

        let ray = Ray::new(Point3::new(-2.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0));

        let result = capsule.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 2.0);
        assert_relative_eq!(result.tmax, 2.0);
    }

    #[test]
    fn intersect_t_axis_aligned_capsule_hit_center() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, -1.0, 0.0),
            p2: Vector3::new(0.0,  1.0, 0.0),
            radius: 0.5,
        };

        let ray = Ray::new(Point3::new(-2.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));

        let result = capsule.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.5);
        assert_relative_eq!(result.tmax, 2.5);
    }

    #[test]
    fn intersect_t_axis_aligned_capsule_miss_parallel() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, -1.0, 0.0),
            p2: Vector3::new(0.0,  1.0, 0.0),
            radius: 0.5,
        };

        let ray = Ray::new(Point3::new(-2.0, 2.0, 0.0), Vector3::new(1.0, 0.0, 0.0));

        assert!(capsule.intersect_t(&ray).is_none());
    }

    #[test]
    fn intersect_t_axis_aligned_capsule_graze_cylinder() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, -1.0, 0.0),
            p2: Vector3::new(0.0,  1.0, 0.0),
            radius: 1.0,
        };

        let ray = Ray::new(Point3::new(-1.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0));

        let result = capsule.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_t_axis_aligned_capsule_hit_endcap() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, -1.0, 0.0),
            p2: Vector3::new(0.0, 1.0, 0.0),
            radius: 1.0,
        };

        let ray = Ray::new(Point3::new(-2.0, 1.5, 0.0), Vector3::new(1.0, 0.0, 0.0));

        let result = capsule.intersect_t(&ray).unwrap();
        assert_relative_eq!(result.tmin, 2.0 - 0.75_f32.sqrt());
        assert_relative_eq!(result.tmax, 2.0 + 0.75_f32.sqrt());
    }

    #[test]
    fn intersect_t_diagonal_capsule_hit() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, 0.0, 0.0),
            p2: Vector3::new(1.0, 1.0, 0.0),
            radius: 0.25,
        };

        let ray = Ray::new(Point3::new(0.5, -1.0, 0.0), Vector3::new(0.0, 1.0, 0.0));

        let result = capsule.intersect_t(&ray).unwrap();
        assert!(result.tmin <= result.tmax);
    }

    #[test]
    fn intersect_t_diagonal_capsule_miss() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, 0.0, 0.0),
            p2: Vector3::new(1.0, 1.0, 0.0),
            radius: 0.25,
        };

        let ray = Ray::new(Point3::new(2.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));

        assert!(capsule.intersect_t(&ray).is_none());
    }

    #[test]
    fn intersect_t_inside_capsule() {
        let capsule = Capsule {
            p1: Vector3::new(0.0, -1.0, 0.0),
            p2: Vector3::new(0.0,  1.0, 0.0),
            radius: 1.0,
        };

        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));

        let result = capsule.intersect_t(&ray).unwrap();
        assert!(result.tmin <= 0.0);
        assert!(result.tmax > 0.0);
    }

}