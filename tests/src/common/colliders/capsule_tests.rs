#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use common::collision::collidable::NarrowPhaseCollisionQuery;
    use common::collision::colliders::capsule::Capsule;
    use common::maths::{Local, Ray};
    use common::engine::resources::Resources;
    use nalgebra::{Point3, Vector3};

    #[test]
    fn intersect_zero_length_origin_capsule_hit() {
        let capsule = Capsule {
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        };

        let ray = Local(Ray::new(Point3::new(-2.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)));

        let result = capsule.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 3.0);
    }

    #[test]
    fn intersect_zero_length_origin_capsule_miss() {
        let capsule = Capsule {
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        };

        let ray = Local(Ray::new(Point3::new(-2.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)));

        let result = capsule.narrow_intersect(&ray, &Resources::new());
        assert!(result.is_none());
    }

    #[test]
    fn intersect_zero_length_origin_capsule_graze() {
        let capsule = Capsule {
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        };

        let ray = Local(Ray::new(Point3::new(-2.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0)));

        let result = capsule.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 2.0);
        assert_relative_eq!(result.tmax, 2.0);
    }

    #[test]
    fn intersect_axis_aligned_capsule_hit_center() {
        let capsule = Capsule {
            p1: Point3::new(0.0, -1.0, 0.0),
            p2: Point3::new(0.0, 1.0, 0.0),
            radius: 0.5,
        };

        let ray = Local(Ray::new(Point3::new(-2.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)));

        let result = capsule.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.5);
        assert_relative_eq!(result.tmax, 2.5);
    }

    #[test]
    fn intersect_axis_aligned_capsule_miss_parallel() {
        let capsule = Capsule {
            p1: Point3::new(0.0, -1.0, 0.0),
            p2: Point3::new(0.0, 1.0, 0.0),
            radius: 0.5,
        };

        let ray = Local(Ray::new(Point3::new(-2.0, 2.0, 0.0), Vector3::new(1.0, 0.0, 0.0)));

        assert!(capsule.narrow_intersect(&ray, &Resources::new()).is_none());
    }

    #[test]
    fn intersect_axis_aligned_capsule_graze_cylinder() {
        let capsule = Capsule {
            p1: Point3::new(0.0, -1.0, 0.0),
            p2: Point3::new(0.0, 1.0, 0.0),
            radius: 1.0,
        };

        let ray = Local(Ray::new(Point3::new(-1.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0)));

        let result = capsule.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_axis_aligned_capsule_hit_endcap() {
        let capsule = Capsule {
            p1: Point3::new(0.0, -1.0, 0.0),
            p2: Point3::new(0.0, 1.0, 0.0),
            radius: 1.0,
        };

        let ray = Local(Ray::new(Point3::new(-2.0, 1.5, 0.0), Vector3::new(1.0, 0.0, 0.0)));

        let result = capsule.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 2.0 - 0.75_f32.sqrt());
        assert_relative_eq!(result.tmax, 2.0 + 0.75_f32.sqrt());
    }

    #[test]
    fn intersect_diagonal_capsule_hit() {
        let capsule = Capsule {
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(1.0, 1.0, 0.0),
            radius: 0.25,
        };

        let ray = Local(Ray::new(Point3::new(0.5, -1.0, 0.0), Vector3::new(0.0, 1.0, 0.0)));

        let result = capsule.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert!(result.tmin <= result.tmax);
    }

    #[test]
    fn intersect_diagonal_capsule_miss() {
        let capsule = Capsule {
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(1.0, 1.0, 0.0),
            radius: 0.25,
        };

        let ray = Local(Ray::new(Point3::new(2.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)));

        assert!(capsule.narrow_intersect(&ray, &Resources::new()).is_none());
    }

    #[test]
    fn intersect_inside_capsule() {
        let capsule = Capsule {
            p1: Point3::new(0.0, -1.0, 0.0),
            p2: Point3::new(0.0, 1.0, 0.0),
            radius: 1.0,
        };

        let ray = Local(Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)));

        let result = capsule.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert!(result.tmin <= 0.0);
        assert!(result.tmax > 0.0);
    }
}
