#[cfg(test)]
mod tests {
    use crate::util;
    use approx::assert_relative_eq;
    use common::collision::collidable::Sweep;
    use common::collision::colliders::sphere::Sphere;
    use common::engine::resources::Resources;
    use common::world::physics_context::ColliderSet;
    use common::world::{Collider, World, WorldNode};
    use nalgebra::{Point3, Vector3};
    use petgraph::prelude::NodeIndex;

    fn setup_plane_test() -> (World, Resources, NodeIndex) {
        let mut resources = Resources::new();

        let plane_handle = util::load_test_geometry_handle("files/big-subdivided-plane.glb".as_ref(), &mut resources);

        let mut world = World::default();
        let plane_node = world.graph.add_node(WorldNode::default());

        world
            .physics_context
            .colliders
            .insert(plane_node, ColliderSet::narrow_only(Collider::Geometry(plane_handle)));

        (world, resources, plane_node)
    }

    #[test]
    fn test_spherecast_perpendicular_graze() {
        let (world, resources, plane_node) = setup_plane_test();

        let sphere = Sphere::new(Point3::new(0.0, 5.0, 0.0), 1.0);
        let velocity = Vector3::new(0.0, -4.0, 0.0);

        let hit_node = world.spherecast(&Sweep::new(sphere, velocity), &resources).unwrap();

        assert_eq!(hit_node.node, plane_node);
        assert_relative_eq!(hit_node.hit.t, 1.0);
        assert_relative_eq!(hit_node.hit.point, Point3::origin());
        assert_relative_eq!(hit_node.hit.normal, Vector3::y_axis());
    }

    #[test]
    fn test_spherecast_perpendicular_fast() {
        let (world, resources, plane_node) = setup_plane_test();

        let sphere = Sphere::new(Point3::new(0.0, 5.0, 0.0), 1.0);
        let velocity = Vector3::new(0.0, -8.0, 0.0);

        let hit_node = world.spherecast(&Sweep::new(sphere, velocity), &resources).unwrap();

        assert_eq!(hit_node.node, plane_node);
        assert_relative_eq!(hit_node.hit.t, 0.5);
        assert_relative_eq!(hit_node.hit.point, Point3::origin());
        assert_relative_eq!(hit_node.hit.normal, Vector3::y_axis());
    }
}
