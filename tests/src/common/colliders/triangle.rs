#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use common::collision::collidable::{NarrowPhaseCollisionQuery, Sweep};
    use common::collision::colliders::sphere::Sphere;
    use common::collision::colliders::triangle::Triangle;
    use common::maths::{Local, Ray};
    use common::resources::Resources;
    use nalgebra::{Point3, Vector3};

    const EPSILON: f32 = 1e-6;

    #[test]
    fn intersect_triangle_perpendicular() {
        let ray = Local(Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)));

        let triangle = Triangle([
            Point3::new(-1.0, -1.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(-1.0, 1.0, 1.0),
        ]);

        let result = triangle.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_triangle_corner() {
        let ray = Local(Ray::new(Point3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)));

        let triangle = Triangle([
            Point3::new(1.0, 0.0, 1.0),
            Point3::new(-1.0, 1.0, 1.0),
            Point3::new(-1.0, -1.0, 1.0),
        ]);

        let result = triangle.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_triangle_edge() {
        let v0 = Point3::new(1.0, 0.0, 1.0);
        let v1 = Point3::new(-1.0, 1.0, 1.0);
        let v2 = Point3::new(-1.0, -1.0, 1.0);

        let v0v1 = v1 - v0;
        let midpoint = v0 + v0v1 / 2.0 - Vector3::new(0.0, 0.0, 1.0);

        let ray = Local(Ray::new(midpoint.into(), Vector3::new(0.0, 0.0, 1.0)));

        //   ^
        //  <->
        // <--->
        let triangle = Triangle([v0, v1, v2]);

        let result = triangle.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_triangle_diagonal() {
        let ray = Local(Ray::new(
            Point3::new(-1.0, -1.0, -1.0),
            Vector3::new(1.0, 1.0, 1.0).normalize(),
        ));

        let triangle = Triangle([
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(-1.0, 1.0, 0.0),
            Point3::new(-1.0, -1.0, 0.0),
        ]);

        let result = triangle.narrow_intersect(&ray, &Resources::new()).unwrap();
        assert_relative_eq!(result.tmin, 3.0_f32.sqrt());
        assert_relative_eq!(result.tmax, 3.0_f32.sqrt());
    }

    #[test]
    fn sweep_intersect_sphere_perpendicular_face_direct() {
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(0.0, 3.0, 0.0), 1.0),
            velocity: Vector3::new(0.0, -2.0, 0.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(-1.0, 0.0, -1.0),
            Point3::new(1.0, 0.0, -1.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new()).unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0));
        assert_relative_eq!(result.t, 1.0);
        assert_relative_eq!(result.normal, Vector3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn sweep_intersect_sphere_perpendicular_face_fast() {
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(0.0, 3.0, 0.0), 1.0),
            velocity: Vector3::new(0.0, -10.0, 0.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(-1.0, 0.0, -1.0),
            Point3::new(1.0, 0.0, -1.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new()).unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0));
        assert_relative_eq!(result.t, 0.2);
        assert_relative_eq!(result.normal, Vector3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn sweep_intersect_sphere_angled_face_direct() {
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(2.0, 2.0, 0.0), 1.0),
            velocity: Vector3::new(-1.0, -1.0, 0.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 2.0),
            Point3::new(-2.0, 0.0, -2.0),
            Point3::new(2.0, 0.0, -2.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new()).unwrap();
        assert_relative_eq!(result.point, Point3::new(1.0, 0.0, 0.0), epsilon = EPSILON);
        assert_relative_eq!(result.t, 1.0, epsilon = EPSILON);
        assert_relative_eq!(result.normal, Vector3::new(0.0, 1.0, 0.0), epsilon = EPSILON);
    }

    #[test]
    fn sweep_intersect_sphere_angled_face_fast() {
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(2.0, 2.0, 0.0), 1.0),
            velocity: Vector3::new(-2.0, -2.0, 0.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 2.0),
            Point3::new(-2.0, 0.0, -2.0),
            Point3::new(2.0, 0.0, -2.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new()).unwrap();
        assert_relative_eq!(result.point, Point3::new(1.0, 0.0, 0.0), epsilon = EPSILON);
        assert_relative_eq!(result.t, 0.5);
        assert_relative_eq!(result.normal, Vector3::new(0.0, 1.0, 0.0), epsilon = EPSILON);
    }

    #[test]
    fn sweep_intersect_sphere_perpendicular_edge_direct() {
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(-2.0, 0.0, 0.0), 1.0),
            velocity: Vector3::new(1.0, 0.0, 0.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new()).unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0));
        assert_relative_eq!(result.t, 1.0);
        assert_relative_eq!(result.normal, Vector3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn sweep_intersect_sphere_angled_edge_direct() {
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(-2.0, 0.0, -2.0), 1.0),
            velocity: Vector3::new(2.0 - 2.0_f32.sqrt() / 2.0, 0.0, 2.0 - 2.0_f32.sqrt() / 2.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, -2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new()).unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0));
        assert_relative_eq!(result.t, 1.0);
        assert_relative_eq!(result.normal, Vector3::new(-0.5_f32.sqrt(), 0.0, -0.5_f32.sqrt()));
    }

    #[test]
    fn sweep_intersect_sphere_vertex_direct() {
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(-2.0, 0.0, 0.0), 1.0),
            velocity: Vector3::new(1.0, 0.0, 0.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new()).unwrap();
        assert_relative_eq!(result.point, Point3::new(0.0, 0.0, 0.0), epsilon = EPSILON);
        assert_relative_eq!(result.t, 1.0);
        assert_relative_eq!(result.normal, Vector3::new(-1.0, 0.0, 0.0), epsilon = EPSILON);
    }

    #[test]
    fn sweep_intersect_sphere_vertex_diagonal() {
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(-2.0, -2.0, 0.0), 1.0),
            velocity: Vector3::new(2.0 - 2.0_f32.sqrt() / 2.0, 2.0 - 2.0_f32.sqrt() / 2.0, 0.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new()).unwrap();
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
        let sweep = Local(Sweep {
            object: Sphere::new(Point3::new(-2.0, -2.0, 0.0), 0.5),
            velocity: Vector3::new(1.0, 1.0, 0.0),
        });

        let triangle = Triangle([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ]);

        let result = triangle.narrow_intersect(&sweep, &Resources::new());
        assert!(result.is_none());
    }
}
