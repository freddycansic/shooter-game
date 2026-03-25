use crate::ecs::component::Component;
use common::ecs::archetype::{Archetype, Column};
use common::ecs::component::StableComponentId;
use common::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use std::marker::PhantomData;

/// The parameter of `Query`, a mixture of immutable and mutable references to `Component`s
pub trait ComponentQuery<'w> {
    type Item;

    fn ids() -> Vec<StableComponentId>;
    unsafe fn fetch(columns: &[*const Column], index: usize) -> Option<Self::Item>;
}

impl<'w, A: Component + 'static> ComponentQuery<'w> for &A {
    type Item = &'w A;

    fn ids() -> Vec<StableComponentId> {
        vec![A::ID]
    }

    unsafe fn fetch(columns: &[*const Column], index: usize) -> Option<&'w A> {
        unsafe { columns[0].as_ref() }
            .unwrap()
            .as_type_ref_unchecked::<A>()
            .get(index)
    }
}

pub struct Query<'a, T: ComponentQuery<'a>> {
    pub archetypes: Vec<&'a mut Archetype>,
    // Query depends on T, but doesn't actually contain a reference to it.
    // So this is here to keep the compiler happy.
    _marker: PhantomData<T>,
}

impl<'a, T: ComponentQuery<'a>> Query<'a, T> {
    fn new(archetypes: Vec<&'a mut Archetype>) -> Self {
        Self {
            archetypes,
            _marker: PhantomData,
        }
    }
}

impl<T: for<'w> ComponentQuery<'w> + 'static> SystemParameter for Query<'_, T> {
    type Item<'w> = Query<'w, T>;

    fn get(world: &mut World) -> Self::Item<'_> {
        Query::new(world.find_superset_archetypes(&T::ids()))
    }
}

impl<'w, T: ComponentQuery<'w>> Query<'w, T> {
    pub fn iter(&'w self) -> QueryIterator<'w, T> {
        QueryIterator {
            query: self,
            archetype_index: 0,
            component_index: 0,
            component_ids: vec![],
            archetype_columns: vec![],
        }
    }
}

pub struct QueryIterator<'w, T: ComponentQuery<'w> + 'static> {
    query: &'w Query<'w, T>,
    archetype_index: usize,
    component_index: usize,
    component_ids: Vec<StableComponentId>,
    archetype_columns: Vec<*const Column>,
}

impl<'q, 'w, T> Iterator for QueryIterator<'w, T>
where
    T: ComponentQuery<'w> + 'static,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.component_ids.is_empty() {
                self.component_ids = T::ids();
            }

            if self.archetype_index >= self.query.archetypes.len() {
                return None;
            }

            if self.archetype_columns.is_empty() {
                self.archetype_columns =
                    self.query.archetypes[self.archetype_index].columns_from_ids(&self.component_ids);
            }

            let result = unsafe { T::fetch(&self.archetype_columns, self.component_index) };

            if let Some(components) = result {
                self.component_index += 1;

                return Some(components);
            } else {
                self.archetype_index += 1;
                self.component_index = 0;
                self.archetype_columns.clear();
            }
        }
    }
}
