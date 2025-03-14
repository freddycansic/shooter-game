use cgmath::Vector3;

use crate::colliders::collider::Collider;

#[derive(Clone)]
pub struct AABBCollider {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Collider for AABBCollider {
    fn colliding(&self, other: &AABBCollider) -> bool {
        self.min.x <= other.max.x
            && self.min.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}
