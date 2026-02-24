use crate::ecs::archetype::Archetype;
use crate::ecs::entity::Entity;
use crate::engine::resources::Resources;
use crate::serde::SerializeWithContext;
use serde::{Deserialize, Serialize};
use crate::ecs::component::StableComponentId;

#[derive(Serialize, Deserialize)]
pub struct SerializedArchetype {
    id: u32,
    entities: Vec<Entity>,
    components: Vec<StableComponentId>,
}

impl SerializeWithContext for Archetype {
    type Serialized = SerializedArchetype;

    fn serialize_with(&self, resources: &Resources) -> Self::Serialized {
        Self::Serialized {
            id: self.id,
            entities: self.entities.clone(),
        }
    }
}
