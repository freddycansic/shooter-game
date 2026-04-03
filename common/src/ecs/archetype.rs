use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use crate::ecs::stable_id::StableId;
use common::ecs::owned_components::OwnedComponents;
use std::any::Any;
use std::cell::OnceCell;

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
    pub id: StableId,
    pub components: OnceCell<Box<dyn ComponentColumn>>,
}

impl Column {
    pub fn new_empty(id: StableId) -> Self {
        Self {
            id,
            components: OnceCell::new(),
        }
    }

    pub fn as_type_ref_unchecked<T: 'static + Component>(&self) -> &Vec<T> {
        debug_assert_eq!(T::ID, self.id);

        self.components
            .get_or_init(|| Box::new(Vec::<T>::new()))
            .as_any_ref()
            .downcast_ref::<Vec<T>>()
            .unwrap()
    }

    pub fn as_type_mut_unchecked<T: 'static + Component>(&mut self) -> &mut Vec<T> {
        debug_assert_eq!(T::ID, self.id);

        self.components
            .get_mut_or_init(|| Box::new(Vec::<T>::new()))
            .as_any_mut()
            .downcast_mut::<Vec<T>>()
            .unwrap()
    }
}

pub struct Archetype {
    pub id: u64,
    pub entities: Vec<Entity>,
    pub columns: Vec<Column>,
}

impl Archetype {
    pub fn spawn<T: OwnedComponents>(&mut self, components: T) -> Entity {
        let entity = Entity {
            archetype_id: self.id,
            row: self.entities.len() as u32,
        };

        components.spawn(self);

        self.entities.push(entity.clone());

        entity
    }

    pub fn matching_columns(&mut self, query_ids: &[StableId]) -> Option<Vec<*mut Column>> {
        let mut matching_columns = Vec::<*mut Column>::with_capacity(query_ids.len());

        for query_id in query_ids {
            if let Ok(index) = self.columns.binary_search_by(|col| col.id.cmp(query_id)) {
                matching_columns.push(&mut self.columns[index] as *mut Column);
            } else {
                return None;
            }
        }

        Some(matching_columns)
    }

    pub fn column_for_id<T: Component>(&self) -> Option<&Column> {
        self.columns.iter().find(|column| column.id == T::ID)
    }

    pub fn column_for_id_mut<T: Component>(&mut self) -> Option<&mut Column> {
        self.columns.iter_mut().find(|column| column.id == T::ID)
    }

    // pub fn build_query_ptr<'w, T>(&mut self, query_ids: &[StableComponentId]) -> Vec<T::QueryPtr>
    // where
    //     T: ComponentQuery<'w>,
    // {
    //     let mut columns = Vec::new();
    //     for component_id in query_ids {
    //         match self.columns.iter_mut().find(|column| column.id == *component_id) {
    //             Some(column) => columns.push(T::query_ptr(column)),
    //             None => {
    //                 log::warn!("Component id {:?} does not exist on archetype", component_id);
    //                 return vec![];
    //             }
    //         }
    //     }
    //
    //     columns
    // }
    //
    // pub fn columns_from_ids_mut<'w, T>(&mut self, query_ids: &[StableComponentId]) -> Vec<T::ColumnPtr>
    // where
    //     T: ComponentQuery<'w, ColumnPtr = *mut Column>,
    // {
    //     let mut columns = Vec::new();
    //     for component_id in query_ids {
    //         match self.columns.iter_mut().find(|column| column.id == *component_id) {
    //             Some(column) => columns.push(column as *mut Column),
    //             None => {
    //                 log::warn!("Component id {:?} does not exist on archetype", component_id);
    //                 return vec![];
    //             }
    //         }
    //     }
    //
    //     columns
    // }
}
