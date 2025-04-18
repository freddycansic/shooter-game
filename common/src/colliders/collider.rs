use cgmath::Vector3;

pub trait Collider {
    fn colliding(&self, other: &Self) -> bool;
}
