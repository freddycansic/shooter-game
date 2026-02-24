use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entity {
    pub archetype_id: u32,
    pub row: u32,
}