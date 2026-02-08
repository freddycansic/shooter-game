use nalgebra::Vector3;
use petgraph::prelude::NodeIndex;
use crate::collision::collidable::{Intersectable, RayHit, SweepHit, RayHitNode, SweepHitNode};
use crate::collision::colliders::sphere::Sphere;
use crate::maths::Ray;
use crate::resources::{GeometryHandle, Resources};
use crate::world::{WorldGraph};

pub enum ColliderType {
    Geometry(GeometryHandle),
    Sphere(Sphere),
}

pub struct Collider {
    pub node: NodeIndex,
    pub ty: ColliderType,
}

pub struct Physics {
    pub colliders: Vec<Collider>
}

impl Physics {
    pub fn intersect_ray(&self, ray: &Ray, world_graph: &WorldGraph, resources: &mut Resources) -> Option<RayHitNode> {
        self.colliders.iter().filter_map(|collider| {
            // collider is in local space, ray is in world space
            let node = world_graph.graph.node_weight(collider.node)?;

            let local_ray = {
                let world_inverse = node.world_transform().matrix().try_inverse().unwrap();

                let local_origin = world_inverse.transform_point(&ray.origin);
                let local_direction = world_inverse.transform_vector(&ray.direction()).normalize();

                Ray::new(local_origin, local_direction)
            };

            match &collider.ty {
                ColliderType::Geometry(geometry_handle) => {
                    let geometry = resources.get_geometry(*geometry_handle);
                    geometry.bvh.intersect_ray(&local_ray)
                }
                ColliderType::Sphere(sphere) => {
                    // sphere.intersect_ray(&local_ray)
                    unimplemented!()
                }
            }.map(|hit| (hit, local_ray, node, collider.node))
        }).min_by(|(a, _, _, _), (b, _, _, _)| a.tmin.partial_cmp(&b.tmin).unwrap()).map(|(local_hit, local_ray, node, node_index)| {
            let world_dir = node.world_transform().matrix().transform_vector(&local_ray.direction());
            let scale = world_dir.magnitude();

            RayHitNode {
                hit: RayHit {
                    tmin: local_hit.tmin * scale,
                    tmax: local_hit.tmax * scale,
                },
                node: node_index,
            }
        })
    }

    pub fn sweep_intersect_sphere(&self, sphere: &Sphere, velocity: &Vector3<f32>, world_graph: &WorldGraph, resources: &mut Resources) -> Option<SweepHitNode> {
        self.colliders.iter().filter_map(|collider| {
            let node = world_graph.graph.node_weight(collider.node)?;

            let world_inverse = node.world_transform().matrix().try_inverse().unwrap();

            let local_sphere = {
                let local_origin = world_inverse.transform_point(&sphere.origin);

                Sphere::new(local_origin, sphere.radius)
            };

            let local_velocity = world_inverse.transform_vector(&velocity);

            match &collider.ty {
                ColliderType::Geometry(geometry_handle) => {
                    let geometry = resources.get_geometry(*geometry_handle);

                    geometry.bvh.sweep_intersect_sphere(&local_sphere, &local_velocity)
                }
                ColliderType::Sphere(sphere) => {
                    // sphere.intersect_ray(&local_ray)
                    unimplemented!()
                }
            }.map(|hit| (hit, local_velocity, node, collider.node))
        }).min_by(|(a, _, _, _), (b, _, _, _)| a.t.partial_cmp(&b.t).unwrap()).map(|(local_hit, local_velocity, node, node_index)| {
            let world_velocity = node.world_transform().matrix().transform_vector(&local_velocity);
            let scale = world_velocity.magnitude();
            let world_t = local_hit.t * scale;

            let world_point = node.world_transform().matrix().transform_point(&local_hit.point);

            let world_normal = node.world_transform().rotation().transform_vector(&local_hit.normal).normalize();

            SweepHitNode {
                hit: SweepHit {
                    t: world_t,
                    point: world_point,
                    normal: world_normal,
                },
                node: node_index,
            }
        })
    }
}