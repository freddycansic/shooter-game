use crate::ecs::component::{Component, Components, StableComponentId};
use crate::ecs::entity::Entity;
use std::any::Any;

pub trait ComponentColumn {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_any_ref(&self) -> &dyn Any;
}

impl<T: 'static> ComponentColumn for Vec<T> {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

pub struct Column {
    pub id: StableComponentId,
    pub components: Option<Box<dyn ComponentColumn>>,
}

impl Column {
    pub fn new_empty(id: StableComponentId) -> Self {
        Self {
            id, components: None
        }
    }
    
    pub fn as_type_ref<T: 'static>(&self) -> Option<&Vec<T>> {
        self.components.as_ref()?.as_any_ref().downcast_ref::<Vec<T>>()
    }

    pub fn as_type_mut<T: 'static>(&mut self) -> Option<&mut Vec<T>> {
        self.components.as_mut()?.as_any_mut().downcast_mut::<Vec<T>>()
    }
}

pub struct Archetype {
    pub id: u64,
    pub entities: Vec<Entity>,
    pub columns: Vec<Column>,
}

impl Archetype {
    pub fn spawn<T: Components>(&mut self, components: T) -> Entity {
        let entity = Entity {
            archetype_id: self.id,
            row: self.entities.len() as u32,
        };

        components.spawn(self);

        self.entities.push(entity.clone());

        entity
    }

    pub fn components_of_type<T: Component + 'static>(&self) -> Option<&Vec<T>> {
        let column = self.columns.iter().find(|column| column.id == T::ID)?;
        column.as_type_ref::<T>()
    }

    pub fn components_of_type_mut<T: Component + 'static>(&mut self) -> Option<&mut Vec<T>> {
        let column = self.columns.iter_mut().find(|column| column.id == T::ID)?;
        column.as_type_mut()
    }
}
