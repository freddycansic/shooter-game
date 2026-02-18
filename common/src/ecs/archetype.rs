use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use std::any::Any;

pub struct Archetype {
    id: u32,
    entities: Vec<Entity>,
    // Any here must be Vec<T: Component>
    components: Vec<(u32, Box<dyn Any>)>,
}

impl Archetype {
    // TODO pub fn push_entity
    
    pub fn components<T: Component + 'static>(&self) -> Option<&[T]> {
        let id = T::id();

        let (_, column) = self.components.iter().find(|(component_id, _)| *component_id == id)?;

        column.downcast_ref::<Vec<T>>().map(|data| data.as_slice())
    }
}
