use crate::collision::collidable::RayHit;
use crate::maths::Ray;
use nalgebra::Vector3;

pub fn intersect_ray(ray: &Ray, p1: &Vector3<f32>, p2: &Vector3<f32>, radius: f32) -> Option<RayHit> {
    let line_segment = p2 - p1;

    let length = line_segment.norm();

    let unit_segment = line_segment / length;
    let p1_to_ray_origin = (ray.origin - p1).to_homogeneous().xyz();
    let d_perp = ray.direction() - unit_segment * ray.direction().dot(&unit_segment);
    let ao_perp = p1_to_ray_origin - unit_segment * p1_to_ray_origin.dot(&unit_segment);

    let a_q = d_perp.dot(&d_perp);
    let b_q = 2.0 * d_perp.dot(&ao_perp);
    let c_q = ao_perp.dot(&ao_perp) - radius * radius;

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

    (tmin <= tmax).then(|| RayHit { tmin, tmax })
}
