use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use std::any::Any;

trait ColumnTrait: Any {
    fn as_any(&self) -> &dyn Any;
}

// Essentially a type erased Vec<T>
// 'static here means that T does not contain any non 'static references
// Any requires 'static because the type it will be cast into must be independent of references
struct Column<T: Component + 'static>(pub Vec<T>);

impl<T: Component> ColumnTrait for Column<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Archetype {
    id: u32,
    entities: Vec<Entity>,
    components: Vec<(u32, Box<dyn ColumnTrait>)>,
}

impl Archetype {
    pub fn components<T: Component + 'static>(&self) -> Option<&[T]> {
        let id = T::id();

        let (_, column) = self.components.iter().find(|(component_id, c)| *component_id == id)?;

        column
            .as_any()
            .downcast_ref::<Column<T>>()
            .map(|data| data.0.as_slice())
    }
}
