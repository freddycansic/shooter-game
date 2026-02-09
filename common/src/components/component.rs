use crate::collision::colliders::sphere::Sphere;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Component {
    PlayerSpawn,
}

impl Component {
    pub fn name(&self) -> &str {
        match self {
            Component::PlayerSpawn => "Player Spawn",
        }
    }
}
