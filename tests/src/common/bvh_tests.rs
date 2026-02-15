#[cfg(test)]
mod tests {
    use crate::util;
    use approx::assert_relative_eq;
    use common::collision::colliders::aabb::Aabb;
    use common::collision::colliders::bvh::{Bvh, BvhNode};
    use common::collision::colliders::triangle::Triangle;
    use common::geometry::Geometry;
    use nalgebra::Point3;
    use petgraph::visit::Bfs;
    use petgraph::{Direction, EdgeDirection};
    use std::path::PathBuf;

    #[test]
    fn bvh_plane_2_triangles() {
        let bvh = util::load_test_geometry("files/plane-2-tris.glb".as_ref()).bvh;
        let root_node = bvh.graph.node_weight(bvh.root).unwrap();

        match root_node {
            BvhNode::Leaf { triangles, aabb } => {
                assert_eq!(triangles.len(), 2);

                assert_relative_eq!(triangles[0].0[0], Point3::new(-1.0, 0.0, 1.0));
                assert_relative_eq!(triangles[0].0[1], Point3::new(1.0, 0.0, 1.0));
                assert_relative_eq!(triangles[0].0[2], Point3::new(1.0, 0.0, -1.0));

                assert_relative_eq!(triangles[1].0[0], Point3::new(-1.0, 0.0, 1.0));
                assert_relative_eq!(triangles[1].0[1], Point3::new(1.0, 0.0, -1.0));
                assert_relative_eq!(triangles[1].0[2], Point3::new(-1.0, 0.0, -1.0));

                assert_relative_eq!(aabb.min, Point3::new(-1.0, 0.0, -1.0));
                assert_relative_eq!(aabb.max, Point3::new(1.0, 0.0, 1.0));
            }
            BvhNode::Aabb(_) => panic!("Aabb should not be created"),
        }
    }

    #[test]
    fn bvh_partition_check_cube_subdivided() {
        let bvh = util::load_test_geometry("files/cube-1-subdivide.glb".as_ref()).bvh;
        let mut bfs = Bfs::new(&bvh.graph, bvh.root);

        while let Some(node_idx) = bfs.next(&bvh.graph) {
            let node = bvh.graph.node_weight(node_idx).unwrap();

            match node {
                BvhNode::Leaf { triangles, aabb } => {
                    for tri in triangles {
                        for vertex in &tri.0 {
                            assert!(vertex.x - aabb.min.x >= 0.0 && aabb.max.x - vertex.x >= 0.0);
                            assert!(vertex.y - aabb.min.y >= 0.0 && aabb.max.y - vertex.y >= 0.0);
                            assert!(vertex.z - aabb.min.z >= 0.0 && aabb.max.z - vertex.z >= 0.0);
                        }
                    }
                }
                BvhNode::Aabb(parent_aabb) => {
                    for child_index in bvh.graph.neighbors_directed(node_idx, Direction::Outgoing) {
                        let child_node = bvh.graph.node_weight(child_index).unwrap();
                        let child_aabb = match child_node {
                            BvhNode::Leaf { aabb, .. } | BvhNode::Aabb(aabb) => aabb,
                        };

                        assert!(
                            child_aabb.min.x - parent_aabb.min.x >= 0.0 && parent_aabb.max.x - child_aabb.max.x >= 0.0,
                        );
                        assert!(
                            child_aabb.min.y - parent_aabb.min.y >= 0.0 && parent_aabb.max.y - child_aabb.max.y >= 0.0,
                        );
                        assert!(
                            child_aabb.min.z - parent_aabb.min.z >= 0.0 && parent_aabb.max.z - child_aabb.max.z >= 0.0,
                        );
                    }
                }
            }
        }

        let root_node = bvh.graph.node_weight(bvh.root).unwrap();
        let root_aabb = match root_node {
            BvhNode::Leaf { aabb, .. } | BvhNode::Aabb(aabb) => aabb,
        };
        assert_relative_eq!(root_aabb.min, Point3::new(-1.0, -1.0, -1.0));
        assert_relative_eq!(root_aabb.max, Point3::new(1.0, 1.0, 1.0));
    }
}
