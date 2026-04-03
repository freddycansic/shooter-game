use crate::ecs::archetype::Archetype;
use crate::ecs::component::StableId;
use crate::ecs::entity::Entity;
use crate::engine::resources::Resources;
use crate::serde::SerializeWithContext;
use glium::glutin::surface::WindowSurface;
use glium::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SerializedArchetype {
    id: u32,
    entities: Vec<Entity>,
    components: Vec<StableId>,
}

impl SerializeWithContext for Archetype {
    type Serialized = SerializedArchetype;

    fn serialize_with(&self, resources: &Resources) -> Self::Serialized {
        unimplemented!();

        //Self::Serialized {
        //    id: self.id,
        //    entities: self.entities.clone(),
        //}
    }

    fn deserialize_with(
        serialized: Self::Serialized,
        display: &Display<WindowSurface>,
        resources: &mut Resources,
    ) -> Self {
        unimplemented!()
    }
}
