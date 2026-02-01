use crate::collision::collidable::{Intersectable, RayHit, SweepHit};
use crate::collision::colliders::cylinder;
use crate::collision::colliders::sphere::Sphere;
use crate::maths::Ray;
use nalgebra::{Point3, Vector3};

#[derive(Debug)]
pub struct Triangle(pub [Point3<f32>; 3]);

impl Triangle {
    pub fn plane_normal(&self) -> Vector3<f32> {
        (self.0[2] - self.0[0]).cross(&(self.0[1] - self.0[0])).normalize()
    }

    pub fn contains_point(&self, plane_intersection: &Point3<f32>, plane_normal: &Vector3<f32>) -> bool {
        let e0 = (self.0[1] - self.0[0])
            .cross(&(plane_intersection - self.0[0]))
            .dot(&plane_normal);
        let e1 = (self.0[2] - self.0[1])
            .cross(&(plane_intersection - self.0[1]))
            .dot(&plane_normal);
        let e2 = (self.0[0] - self.0[2])
            .cross(&(plane_intersection - self.0[2]))
            .dot(&plane_normal);

        // Accept clockwise or counterclockwise
        (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0) || (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0)
    }

    fn sweep_intersect_sphere_on_face(&self, sphere: &Sphere, velocity: &Vector3<f32>) -> Option<SweepHit> {
        let n = self.plane_normal();

        // Sphere is moving along triangle plane
        let denom = velocity.dot(&n);
        if denom.abs() < f32::EPSILON {
            let dist = (sphere.origin - self.0[0]).dot(&n);

            // no overlap at any time
            if dist.abs() > sphere.radius {
                return None;
            }

            // overlapping for the entire sweep = contact
            let p = sphere.origin - n * dist;

            return self.contains_point(&p, &n).then(|| SweepHit {
                t: 0.0,
                normal: if dist > 0.0 { n } else { -n },
                point: p,
            });
        }

        // Time along velocity where sphere intersects triangle plane
        let t = ((self.0[0] - sphere.origin).dot(&n) + sphere.radius) / denom;

        if t < -f32::EPSILON || t > 1.0 + f32::EPSILON {
            return None;
        }

        let p = sphere.origin + velocity * t - n * sphere.radius;

        self.contains_point(&p, &n).then(|| SweepHit { t, normal: n, point: p })
    }

    fn sweep_intersect_sphere_on_edge(
        p1: &Point3<f32>,
        p2: &Point3<f32>,
        sphere: &Sphere,
        velocity: &Vector3<f32>,
    ) -> Option<SweepHit> {
        let ray = Ray::new(sphere.origin, *velocity);

        let hit = cylinder::intersect_ray(&ray, p1, p2, sphere.radius)?;

        // Reject if outside of the velocity
        if hit.tmin < -f32::EPSILON || hit.tmin > 1.0 + f32::EPSILON {
            return None;
        }

        let origin_at_collision = sphere.origin + velocity * hit.tmin;

        // Find closest point to sphere at point of collision
        let edge_dir = (p2 - p1).normalize();
        let to_point = origin_at_collision - p1;
        let proj = edge_dir * to_point.dot(&edge_dir);
        let closest = p1 + proj;

        let edge_normal = (origin_at_collision - closest).normalize();

        Some(SweepHit {
            t: hit.tmin,
            point: closest,
            normal: edge_normal,
        })
    }

    fn sweep_intersect_sphere_on_vertex(
        vertex: &Point3<f32>,
        sphere: &Sphere,
        velocity: &Vector3<f32>,
    ) -> Option<SweepHit> {
        let m = sphere.origin - vertex;
        let a = velocity.dot(velocity);
        let b = 2.0 * m.dot(velocity);
        let c = m.dot(&m) - sphere.radius * sphere.radius;

        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        }

        let sqrt_disc = disc.sqrt();
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        let t = [t1, t2]
            .into_iter()
            .filter(|&t| t >= 0.0 && t <= 1.0)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;

        let sphere_center_at_collision = sphere.origin + velocity * t;

        // Contact point on sphere surface closest to the vertex
        let normal = (sphere_center_at_collision - vertex).normalize();
        let contact_point = sphere_center_at_collision - normal * sphere.radius;

        Some(SweepHit {
            t,
            point: contact_point,
            normal,
        })
    }
}

impl Intersectable for Triangle {
    fn sweep_intersect_sphere(&self, sphere: &Sphere, velocity: &Vector3<f32>) -> Option<SweepHit> {
        let mut closest_hit = None;

        if let Some(hit) = self.sweep_intersect_sphere_on_face(sphere, velocity) {
            closest_hit = Some(hit);
        }

        for (i, j) in [(0, 1), (1, 2), (2, 0)] {
            if let Some(hit) = Self::sweep_intersect_sphere_on_edge(&self.0[i], &self.0[j], sphere, velocity) {
                closest_hit = match closest_hit {
                    None => Some(hit),
                    Some(closest) => {
                        if hit.t < closest.t {
                            Some(hit)
                        } else {
                            Some(closest)
                        }
                    }
                }
            }
        }

        for i in 0..=2 {
            if let Some(hit) = Self::sweep_intersect_sphere_on_vertex(&self.0[i], sphere, velocity) {
                closest_hit = match closest_hit {
                    None => Some(hit),
                    Some(closest) => {
                        if hit.t < closest.t {
                            Some(hit)
                        } else {
                            Some(closest)
                        }
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        if let Some(hit) = closest_hit.as_ref() {
            assert!(hit.t <= 1.0 + f32::EPSILON);
            assert!(hit.t >= -f32::EPSILON);
        }

        closest_hit
    }

    fn intersect_ray(&self, ray: &Ray) -> Option<RayHit> {
        let n = self.plane_normal();

        let denom = ray.direction().dot(&n);
        // Reject parallel rays and clockwise winding order
        if denom.abs() < f32::EPSILON || denom > 0.0 {
            return None;
        }

        let t = (self.0[0] - ray.origin).dot(&n) / denom;

        if t < 0.0 {
            return None;
        }

        // Plane intersection
        let p = ray.origin + ray.direction() * t;

        self.contains_point(&p, &n).then(|| RayHit { tmin: t, tmax: t })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    const EPSILON: f32 = 1e-6;

    #[test]
    fn intersect_ray_triangle_perpendicular() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));

        let triangle = Triangle([
            Point3::new(-1.0, -1.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(-1.0, 1.0, 1.0),
        ]);

        let result = triangle.intersect_ray(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_ray_triangle_corner() {
        let ray = Ray::new(Point3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));

        let triangle = Triangle([
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(-1.0, 1.0, 1.0),
            Point3::new(-1.0, -1.0, 1.0),
        ]);

        let result = triangle.intersect_ray(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_ray_triangle_edge() {
        let v0 = Point3::new(1.0, 0.0, 1.0);
        let v1 = Point3::new(-1.0, 1.0, 1.0);
        let v2 = Point3::new(-1.0, -1.0, 1.0);

        let v0v1 = v1 - v0;
        let midpoint = v0 + v0v1 / 2.0 - Vector3::new(0.0, 0.0, 1.0);

        let ray = Ray::new(midpoint.into(), Vector3::new(0.0, 0.0, 1.0));

        //   ^
        //  <->
        // <--->
        let triangle = Triangle([v0, v1, v2]);

        let result = triangle.intersect_ray(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_ray_triangle_diagonal() {
        let ray = Ray::new(Point3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0).normalize());

        let triangle = Triangle([
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(-1.0, 1.0, 0.0),
            Point3::new(-1.0, -1.0, 0.0),
        ]);

        let result = triangle.intersect_ray(&ray).unwrap();
        assert_relative_eq!(result.tmin, 3.0_f32.sqrt());
        assert_relative_eq!(result.tmax, 3.0_f32.sqrt());
    }

    #[test]
    fn sweep_intersect_sphere_perpendicular_face_direct() {
        let sphere = Sphere::new(Point3::new(0.0, 3.0, 0.0), 1.0);

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(-1.0, 0.0, -1.0),
            Point3::new(1.0, 0.0, -1.0),
        ]);

        let result = triangle
            .sweep_intersect_sphere(&sphere, &Vector3::new(0.0, -2.0, 0.0))
            .unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0));
        assert_relative_eq!(result.t, 1.0);
        assert_relative_eq!(result.normal, Vector3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn sweep_intersect_sphere_perpendicular_face_fast() {
        let sphere = Sphere::new(Point3::new(0.0, 3.0, 0.0), 1.0);

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(-1.0, 0.0, -1.0),
            Point3::new(1.0, 0.0, -1.0),
        ]);

        let result = triangle
            .sweep_intersect_sphere(&sphere, &Vector3::new(0.0, -10.0, 0.0))
            .unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0));
        assert_relative_eq!(result.t, 0.2);
        assert_relative_eq!(result.normal, Vector3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn sweep_intersect_sphere_angled_face_direct() {
        let sphere = Sphere::new(Point3::new(2.0, 2.0, 0.0), 1.0);

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 2.0),
            Point3::new(-2.0, 0.0, -2.0),
            Point3::new(2.0, 0.0, -2.0),
        ]);

        let result = triangle
            .sweep_intersect_sphere(&sphere, &Vector3::new(-1.0, -1.0, 0.0))
            .unwrap();
        assert_relative_eq!(result.point, Point3::new(1.0, 0.0, 0.0), epsilon = EPSILON);
        assert_relative_eq!(result.t, 1.0, epsilon = EPSILON);
        assert_relative_eq!(result.normal, Vector3::new(0.0, 1.0, 0.0), epsilon = EPSILON);
    }

    #[test]
    fn sweep_intersect_sphere_angled_face_fast() {
        let sphere = Sphere::new(Point3::new(2.0, 2.0, 0.0), 1.0);

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 2.0),
            Point3::new(-2.0, 0.0, -2.0),
            Point3::new(2.0, 0.0, -2.0),
        ]);

        let result = triangle
            .sweep_intersect_sphere(&sphere, &Vector3::new(-2.0, -2.0, 0.0))
            .unwrap();
        assert_relative_eq!(result.point, Point3::new(1.0, 0.0, 0.0), epsilon = EPSILON);
        assert_relative_eq!(result.t, 0.5);
        assert_relative_eq!(result.normal, Vector3::new(0.0, 1.0, 0.0), epsilon = EPSILON);
    }

    #[test]
    fn sweep_intersect_sphere_perpendicular_edge_direct() {
        let sphere = Sphere::new(Point3::new(-2.0, 0.0, 0.0), 1.0);

        let triangle = Triangle([
            Point3::new(0.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ]);

        let result = triangle
            .sweep_intersect_sphere(&sphere, &Vector3::new(1.0, 0.0, 0.0))
            .unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0));
        assert_relative_eq!(result.t, 1.0);
        assert_relative_eq!(result.normal, Vector3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn sweep_intersect_sphere_angled_edge_direct() {
        let sphere = Sphere::new(Point3::new(-2.0, 0.0, -2.0), 1.0);

        let triangle = Triangle([
            Point3::new(0.0, -2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ]);

        let result = triangle
            .sweep_intersect_sphere(
                &sphere,
                &Vector3::new(2.0 - 2.0_f32.sqrt() / 2.0, 0.0, 2.0 - 2.0_f32.sqrt() / 2.0),
            )
            .unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0));
        assert_relative_eq!(result.t, 1.0);
        assert_relative_eq!(result.normal, Vector3::new(-0.5_f32.sqrt(), 0.0, -0.5_f32.sqrt()));
    }

    #[test]
    fn sweep_intersect_sphere_vertex_direct() {
        let sphere = Sphere::new(Point3::new(-2.0, 0.0, 0.0), 1.0);

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ]);

        let result = triangle
            .sweep_intersect_sphere(&sphere, &Vector3::new(1.0, 0.0, 0.0))
            .unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0), epsilon = EPSILON);
        assert_relative_eq!(result.t, 1.0);
        assert_relative_eq!(result.normal, Vector3::new(-1.0, 0.0, 0.0), epsilon = EPSILON);
    }

    #[test]
    fn sweep_intersect_sphere_vertex_diagonal() {
        let sphere = Sphere::new(Point3::new(-2.0, -2.0, 0.0), 1.0);

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ]);

        let result = triangle
            .sweep_intersect_sphere(
                &sphere,
                &Vector3::new(2.0 - 2.0_f32.sqrt() / 2.0, 2.0 - 2.0_f32.sqrt() / 2.0, 0.0),
            )
            .unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0), epsilon = EPSILON);
        assert_relative_eq!(result.t, 1.0, epsilon = EPSILON);
        assert_relative_eq!(
            result.normal,
            Vector3::new(-0.5_f32.sqrt(), -0.5_f32.sqrt(), 0.0),
            epsilon = EPSILON
        );
    }

    #[test]
    fn sweep_intersect_sphere_vertex_miss() {
        let sphere = Sphere::new(Point3::new(-2.0, -2.0, 0.0), 0.5);

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ]);

        let result = triangle.sweep_intersect_sphere(&sphere, &Vector3::new(1.0, 1.0, 0.0));
        assert!(result.is_none());
    }
}
