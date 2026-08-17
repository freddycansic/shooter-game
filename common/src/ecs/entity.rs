use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub archetype_id: u64,
    pub row: u32,
}