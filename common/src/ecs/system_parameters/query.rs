use crate::ecs::component::Component;
use common::ecs::archetype::Column;
use common::ecs::component::StableId;
use common::ecs::system::SystemState;
use common::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use itertools::Itertools;
use std::marker::PhantomData;
use crate::executor::{CommandExecutor, RuntimeExecutor};

/// The parameter of `Query`, a mixture of immutable and mutable references to `Component`s
pub trait ComponentQuery<'w> {
    /// Type of item from iterating the query.
    type Item;

    /// This is a tuple of pointers to columns which match the type of the query.
    type QueryPtr;

    fn unsorted_ids() -> Vec<StableId>;

    fn build_query_ptr(columns: &[*mut Column], cursor: &mut usize) -> Self::QueryPtr;

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item>;
}

impl<'w, A: Component + 'static> ComponentQuery<'w> for &A {
    type Item = &'w A;
    type QueryPtr = *const Column;

    fn unsorted_ids() -> Vec<StableId> {
        vec![A::ID]
    }

    fn build_query_ptr(columns: &[*mut Column], cursor: &mut usize) -> Self::QueryPtr {
        let ptr = columns[*cursor];
        *cursor += 1;
        ptr
    }

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item> {
        unsafe { query_ptr.as_ref() }
            .unwrap()
            .as_type_ref_unchecked::<A>()
            .get(index)
    }
}

impl<'w, A: Component + 'static> ComponentQuery<'w> for &mut A {
    type Item = &'w mut A;
    type QueryPtr = *mut Column;

    fn unsorted_ids() -> Vec<StableId> {
        vec![A::ID]
    }

    fn build_query_ptr(columns: &[*mut Column], cursor: &mut usize) -> Self::QueryPtr {
        let ptr = columns[*cursor];
        *cursor += 1;
        ptr
    }

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item> {
        unsafe { query_ptr.as_mut() }
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
    type QueryPtr = (A::QueryPtr, B::QueryPtr);

    fn unsorted_ids() -> Vec<StableId> {
        let mut ids = A::unsorted_ids();
        ids.extend(B::unsorted_ids());
        ids
    }

    fn build_query_ptr(columns: &[*mut Column], cursor: &mut usize) -> Self::QueryPtr {
        let a = A::build_query_ptr(columns, cursor);
        let b = B::build_query_ptr(columns, cursor);
        (a, b)
    }

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item> {
        let a = unsafe { A::fetch(&query_ptr.0, index) };
        let b = unsafe { B::fetch(&query_ptr.1, index) };

        a.and_then(|a| b.map(|b| (a, b)))
    }
}

pub struct Query<'w, T: ComponentQuery<'w>> {
    pub world: &'w mut World,
    // Query depends on T, but doesn't actually contain a reference to it.
    // So this is here to keep the compiler happy.
    _marker: PhantomData<T>,
}

impl<'a, T: ComponentQuery<'a>> Query<'a, T> {
    fn new(world: &'a mut World) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }
}

impl<T: for<'w> ComponentQuery<'w> + 'static> SystemParameter for Query<'_, T> {
    type Item<'w, 's, 'e> = Query<'w, T>;

    fn get<'w, 's, 'e>(world: &'w mut World, state: &'s mut SystemState, executor: &'e mut dyn CommandExecutor) -> Self::Item<'w, 's, 'e> {
        Query::new(world)
    }
}

impl<'w, T: ComponentQuery<'w>> Query<'w, T> {
    pub fn iter(&mut self) -> impl Iterator<Item = T::Item> {
        let query_ids = T::unsorted_ids();

        let archetype_columns = self.world.find_matching_archetype_columns(&query_ids);

        let matching_archetypes = archetype_columns
            .iter()
            .map(|columns| {
                let mut cursor = 0;
                T::build_query_ptr(&columns, &mut cursor)
            })
            .collect_vec();

        QueryIterator::<T> {
            component_index: 0,
            matching_archetypes,
            archetype_index: 0,
        }
    }
}

pub struct QueryIterator<'w, T: ComponentQuery<'w>> {
    component_index: usize,
    matching_archetypes: Vec<T::QueryPtr>,
    archetype_index: usize,
}

impl<'q, 'w, T> Iterator for QueryIterator<'w, T>
where
    T: ComponentQuery<'w>,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.archetype_index >= self.matching_archetypes.len() {
                return None;
            }

            let result = unsafe { T::fetch(&self.matching_archetypes[self.archetype_index], self.component_index) };

            if result.is_some() {
                self.component_index += 1;

                return result;
            } else {
                self.component_index = 0;
                self.archetype_index += 1;
            }
        }
    }
}
