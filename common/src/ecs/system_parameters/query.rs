use crate::ecs::component::Component;
use common::ecs::archetype::{Archetype, Column};
use common::ecs::component::StableComponentId;
use common::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use itertools::Itertools;
use std::marker::PhantomData;

/// The parameter of `Query`, a mixture of immutable and mutable references to `Component`s
pub trait ComponentQuery<'w> {
    type Item;
    type ColumnPtr;

    fn ids() -> Vec<StableComponentId>;
    unsafe fn fetch(columns: &Self::ColumnPtr, index: usize) -> Option<Self::Item>;
}

impl<'w, A: Component + 'static> ComponentQuery<'w> for &A {
    type Item = &'w A;
    type ColumnPtr = *const Column;

    fn ids() -> Vec<StableComponentId> {
        vec![A::ID]
    }

    unsafe fn fetch(columns: &Self::ColumnPtr, index: usize) -> Option<Self::Item> {
        unsafe { columns.as_ref() }
            .unwrap()
            .as_type_ref_unchecked::<A>()
            .get(index)
    }
}

impl<'w, A: Component + 'static> ComponentQuery<'w> for &mut A {
    type Item = &'w mut A;
    type ColumnPtr = *mut Column;

    fn ids() -> Vec<StableComponentId> {
        vec![A::ID]
    }

    unsafe fn fetch(columns: &Self::ColumnPtr, index: usize) -> Option<Self::Item> {
        unsafe { columns.as_mut() }
            .unwrap()
            .as_type_mut_unchecked::<A>()
            .get_mut(index)
    }
}

impl<'w, A, B> ComponentQuery<'w> for (A, B)
where
    A: ComponentQuery<'w>,
    B: ComponentQuery<'w>,
{
    type Item = (A::Item, B::Item);
    type ColumnPtr = (A::ColumnPtr, B::ColumnPtr);

    fn ids() -> Vec<StableComponentId> {
        let mut ids = vec![A::Item::ID, B::Item::ID];
        ids.sort();
        ids
    }

    unsafe fn fetch(columns: &Self::ColumnPtr, index: usize) -> Option<Self::Item> {
        let a = unsafe { A::fetch(&columns.0, index) };
        let b = unsafe { B::fetch(&columns.1, index) };

        a.and_then(|a| b.map(|b| (a, b)))
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

impl<'w, T: ComponentQuery<'w> + 'w> Query<'w, T> {
    pub fn iter(&self) -> impl Iterator<Item = T::Item> {
        let component_ids = T::ids();
        let matching_columns = self
            .archetypes
            .iter()
            .map(|a| a.columns_from_ids(&component_ids))
            .collect_vec();

        QueryIterator {
            component_index: 0,
            matching_columns,
            column_index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = T::Item> {
        let component_ids = T::ids();
        let matching_columns = self
            .archetypes
            .iter_mut()
            .map(|a| a.columns_from_ids_mut(&component_ids))
            .collect_vec();

        QueryIterator {
            component_index: 0,
            matching_columns,
            column_index: 0,
        }
    }
}

pub struct QueryIterator<'w, T: ComponentQuery<'w>> {
    component_index: usize,
    matching_columns: Vec<Vec<T::ColumnPtr>>,
    column_index: usize,
}

impl<'q, 'w, T> Iterator for QueryIterator<'w, T>
where
    T: ComponentQuery<'w>,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.column_index >= self.matching_columns.len() {
                return None;
            }

            let result = unsafe { T::fetch(&self.matching_columns[self.column_index], self.component_index) };

            if result.is_some() {
                self.component_index += 1;

                return result;
            } else {
                self.component_index = 0;
                self.column_index += 1;
            }
        }
    }
}
