#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use common::collision::collidable::{BroadPhaseCollisionQuery, NarrowPhaseCollisionQuery};
    use common::collision::colliders::aabb::Aabb;
    use common::collision::colliders::capsule::Capsule;
    use common::engine::assets::Assets;
    use common::maths::{Local, Ray};
    use nalgebra::{Point3, Vector3};

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

        let result = aabb.narrow_intersect(&ray, &Assets::new()).unwrap();
        assert_relative_eq!(result.tmin, 3_f32.sqrt());
    }

    #[test]
    fn intersect_aabb_face_hit() {
        let ray = Local(Ray::new(Point3::new(0.0, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Assets::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
    }

    #[test]
    fn intersect_aabb_edge_hit() {
        let ray = Local(Ray::new(Point3::new(0.0, 1.0, 1.0), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Assets::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
    }

    #[test]
    fn intersect_ray_inside_aabb() {
        let ray = Local(Ray::new(Point3::new(1.5, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Assets::new()).unwrap();

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

        assert!(aabb.narrow_intersect(&ray, &Assets::new()).is_none());
    }

    #[test]
    fn intersect_aabb_behind_ray() {
        let ray = Local(Ray::new(Point3::new(3.0, 1.5, 1.5), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        assert!(aabb.narrow_intersect(&ray, &Assets::new()).is_none());
    }

    #[test]
    fn intersect_aabb_grazing_hit() {
        let ray = Local(Ray::new(Point3::new(0.0, 2.0, 1.5), Vector3::new(1.0, 0.0, 0.0)));

        let aabb = Aabb {
            min: Point3::new(1.0, 1.0, 1.0),
            max: Point3::new(2.0, 2.0, 2.0),
        };

        let result = aabb.narrow_intersect(&ray, &Assets::new()).unwrap();
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), false);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), false);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), false);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), false);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
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

        assert_eq!(aabb.broad_intersect(&capsule, &Assets::new()), true);
    }
}
