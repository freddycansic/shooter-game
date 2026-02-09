use crate::collision::collidable::{BroadPhaseCollisionQuery, NarrowPhaseCollisionQuery, RayHit, RayHitNode, Sweep, SweepHit, SweepHitNode};
use crate::collision::colliders::aabb::Aabb;
use crate::collision::colliders::capsule::Capsule;
use crate::collision::colliders::sphere::Sphere;
use crate::maths::{Local, Ray};
use crate::resources::{GeometryHandle, Resources};
use crate::world::{World, WorldGraph};
use fxhash::FxHashMap;
use nalgebra::Vector3;
use petgraph::prelude::NodeIndex;

pub enum Collider {
    Aabb(Aabb),
    Capsule(Capsule),
    Sphere(Sphere),
    Geometry(GeometryHandle),
}

impl BroadPhaseCollisionQuery<Local<Ray>> for Collider {
    fn broad_intersect(&self, query: &Local<Ray>, resources: &Resources) -> bool {
        match self {
            Collider::Aabb(aabb) => aabb.narrow_intersect(query, resources).is_some(),
            Collider::Capsule(capsule) => capsule.narrow_intersect(query, resources).is_some(),
            Collider::Sphere(_sphere) => unimplemented!(),
            Collider::Geometry(_geometry_handle) => {
                log::warn!("Using geometry for broad phase ray collision. Geometry should be used for narrow phase only");
                false
            },
        }
    }
}

impl NarrowPhaseCollisionQuery<Local<Ray>> for Collider {
    type Hit = Option<RayHit>;

    fn narrow_intersect(&self, query: &Local<Ray>, resources: &Resources) -> Self::Hit {
        match self {
            Collider::Aabb(aabb) => aabb.narrow_intersect(query, resources),
            Collider::Capsule(capsule) => capsule.narrow_intersect(query, resources),
            Collider::Sphere(_sphere) => unimplemented!(),
            Collider::Geometry(geometry_handle) => {
                let geometry = resources.get_geometry(*geometry_handle);
                geometry.bvh.narrow_intersect(query, resources)
            },
        }
    }
}

impl BroadPhaseCollisionQuery<Local<Sweep<Sphere>>> for Collider {
    fn broad_intersect(&self, query: &Local<Sweep<Sphere>>, resources: &Resources) -> bool {
        match self {
            Collider::Aabb(aabb) => aabb.broad_intersect(query, resources),
            Collider::Capsule(_capsule) => unimplemented!(),
            Collider::Sphere(_sphere) => unimplemented!(),
            Collider::Geometry(_geometry_handle) => {
                log::warn!("Using geometry for broad phase ray collision. Geometry should be used for narrow phase only");
                false
            }
        }
    }
}

impl NarrowPhaseCollisionQuery<Local<Sweep<Sphere>>> for Collider {
    type Hit = Option<SweepHit>;

    fn narrow_intersect(&self, query: &Local<Sweep<Sphere>>, resources: &Resources) -> Self::Hit {
        match self {
            Collider::Aabb(_aabb) => unimplemented!(),
            Collider::Capsule(_capsule) => unimplemented!(),
            Collider::Sphere(_sphere) => unimplemented!(),
            Collider::Geometry(geometry_handle) => {
                let geometry = resources.get_geometry(*geometry_handle);
                geometry.bvh.narrow_intersect(query, resources)
            }
        }
    }
}

pub struct ColliderSet {
    pub node: NodeIndex,
    pub broad: Option<Collider>,
    pub narrow: Collider,
}

impl ColliderSet {
    // TODO do some cast<T> type shiz to reduce code duplication
    pub fn raycast(&self, local_ray: &Local<Ray>, resources: &Resources) -> Option<RayHit> {
        if let Some(broad_collider) = &self.broad {
            if !broad_collider.broad_intersect(local_ray, resources) {
                return None;
            }
        }

        self.narrow.narrow_intersect(local_ray, resources)
    }

    pub fn spherecast(
        &self,
        query: &Local<Sweep<Sphere>>,
        resources: &Resources,
    ) -> Option<SweepHit> {
        if let Some(broad_collider) = &self.broad {
            if broad_collider.broad_intersect(query, resources) {
                return None;
            }
        }

        self.narrow.narrow_intersect(query, resources)
    }
}

/// This struct owns the physics state of the world
/// It does not do any physics work.
pub struct PhysicsContext {
    pub colliders: FxHashMap<NodeIndex, ColliderSet>,
    // TODO should have a bvh which is built when adding colliders for fast broad phase
    // collider_bvh: Bvh
}

impl PhysicsContext {
    pub fn new() -> Self {
        Self {
            colliders: FxHashMap::default(),
        }
    }

    pub fn raycast(&self, ray: &Ray, world_graph: &WorldGraph, resources: &mut Resources) -> Option<RayHitNode> {
        self.colliders
            .iter()
            .filter_map(|(node_index, collider_set)| {
                let node = world_graph.graph.node_weight(*node_index).unwrap();

                // collider is in local space, ray is in world space
                let local_ray = ray.to_local(&node.world_transform());

                collider_set
                    .raycast(&local_ray, resources)
                    .map(|hit| (hit, local_ray, node, node_index))
            })
            .min_by(|(a, _, _, _), (b, _, _, _)| a.tmin.partial_cmp(&b.tmin).unwrap())
            .map(|(local_hit, local_ray, node, node_index)| {
                let hit_point_local = local_ray.point_at(local_hit.tmin);
                let hit_point_world = node.world_transform().matrix().transform_point(&hit_point_local);
                let tmin_world = (hit_point_world - ray.origin).norm();

                RayHitNode {
                    node: *node_index,
                    hit: RayHit {
                        tmin: tmin_world,
                        tmax: (node
                            .world_transform()
                            .matrix()
                            .transform_point(&local_ray.point_at(local_hit.tmax))
                            - ray.origin)
                            .norm(),
                    },
                }
            })
    }

    pub fn spherecast(
        &self,
        query: &Sweep<Sphere>,
        world_graph: &WorldGraph,
        resources: &mut Resources,
    ) -> Option<SweepHitNode> {
        self.colliders
            .iter()
            .filter_map(|(&node_index, collider_set)| {
                let node = world_graph.graph.node_weight(node_index)?;

                let world_inverse = node.world_transform().matrix().try_inverse().unwrap();

                let local_query = {
                    let local_sphere = Sphere::new(world_inverse.transform_point(&query.object.origin), query.object.radius);
                    let local_velocity = world_inverse.transform_vector(&query.velocity);

                    Local(Sweep {
                        object: local_sphere,
                        velocity: local_velocity,
                    })
                };

                collider_set
                    .spherecast(&local_query, resources)
                    .map(|hit| (hit, node, node_index))
            })
            .min_by(|(a, _, _), (b, _, _)| a.t.partial_cmp(&b.t).unwrap())
            .map(|(local_hit, node, node_index)| {
                let world_point = node.world_transform().matrix().transform_point(&local_hit.point);
                let world_normal = node
                    .world_transform()
                    .rotation()
                    .transform_vector(&local_hit.normal)
                    .normalize();

                let world_t = (world_point - query.object.origin).norm();

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
