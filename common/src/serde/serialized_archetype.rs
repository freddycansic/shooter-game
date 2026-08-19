use crate::ecs::archetype::Archetype;
use crate::ecs::component::StableId;
use crate::ecs::entity::Entity;
use crate::engine::assets::Assets;
use crate::serde::SerializeWithContext;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SerializedArchetype {
    id: u32,
    entities: Vec<Entity>,
    components: Vec<StableId>,
}

impl SerializeWithContext for Archetype {
    type Serialized = SerializedArchetype;

    fn serialize_with(&self, _resources: &Assets) -> Self::Serialized {
        unimplemented!();

        //Self::Serialized {
        //    id: self.id,
        //    entities: self.entities.clone(),
        //}
    }

    fn deserialize_with(
        _serialized: Self::Serialized,
        _display: &Display<WindowSurface>,
        _resources: &mut Assets,
    ) -> Self {
        unimplemented!()
    }
}
