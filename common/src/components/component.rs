use serde::{Deserialize, Serialize};
use crate::collision::colliders::sphere::Sphere;

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