use crate::ecs::component::Component;
use crate::runtime::ApplicationAccess;
use common::ecs::archetype::Column;
use common::ecs::component::StableId;
use common::ecs::system::SystemState;
use common::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use itertools::Itertools;
use std::marker::PhantomData;

pub enum ArgumentRequirement {
    Required,
    Optional,
}

pub struct QueryArgument {
    pub requirement: ArgumentRequirement,
    pub component_id: StableId,
}

impl QueryArgument {
    pub fn required<T: Component>() -> Self {
        Self {
            requirement: ArgumentRequirement::Required,
            component_id: T::ID,
        }
    }

    pub fn optional<T: Component>() -> Self {
        Self {
            requirement: ArgumentRequirement::Optional,
            component_id: T::ID,
        }
    }
}

/// The parameter of `Query`, a mixture of immutable and mutable references to `Component`s
pub trait QueryParameter<'w> {
    /// Type of item from iterating the query.
    type Item;

    /// This is a tuple of pointers to columns which match the type of the query.
    type QueryPtr;

    fn query_arguments() -> Vec<QueryArgument>;

    fn build_query_ptr(columns: &[Option<*mut Column>], cursor: &mut usize) -> Self::QueryPtr;

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item>;
}

impl<'w, A: Component + 'static> QueryParameter<'w> for &A {
    type Item = &'w A;
    type QueryPtr = *const Column;

    fn query_arguments() -> Vec<QueryArgument> {
        vec![QueryArgument::required::<A>()]
    }

    fn build_query_ptr(columns: &[Option<*mut Column>], cursor: &mut usize) -> Self::QueryPtr {
        let ptr = columns[*cursor];
        *cursor += 1;
        ptr.unwrap()
    }

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item> {
        unsafe { query_ptr.as_ref() }
            .unwrap()
            .as_type_ref_unchecked::<A>()
            .get(index)
    }
}

impl<'w, A: Component + 'static> QueryParameter<'w> for &mut A {
    type Item = &'w mut A;
    type QueryPtr = *mut Column;

    fn query_arguments() -> Vec<QueryArgument> {
        vec![QueryArgument::required::<A>()]
    }

    fn build_query_ptr(columns: &[Option<*mut Column>], cursor: &mut usize) -> Self::QueryPtr {
        let ptr = columns[*cursor];
        *cursor += 1;
        ptr.unwrap()
    }

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item> {
        unsafe { query_ptr.as_mut() }
            .unwrap()
            .as_type_mut_unchecked::<A>()
            .get_mut(index)
    }
}

impl<'w, A: Component + 'static> QueryParameter<'w> for Option<&A> {
    type Item = Option<&'w A>;
    type QueryPtr = Option<*const Column>;

    fn query_arguments() -> Vec<QueryArgument> {
        vec![QueryArgument::optional::<A>()]
    }

    fn build_query_ptr(columns: &[Option<*mut Column>], cursor: &mut usize) -> Self::QueryPtr {
        let ptr = columns[*cursor];
        *cursor += 1;

        match ptr {
            Some(ptr) => Some(ptr as *const Column),
            None => None,
        }
    }

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item> {
        Some(query_ptr.and_then(|ptr| unsafe { ptr.as_ref().unwrap().as_type_ref_unchecked::<A>().get(index) }))
    }
}

impl<'w, A, B> QueryParameter<'w> for (A, B)
where
    A: QueryParameter<'w>,
    B: QueryParameter<'w>,
{
    type Item = (A::Item, B::Item);
    type QueryPtr = (A::QueryPtr, B::QueryPtr);

    fn query_arguments() -> Vec<QueryArgument> {
        let mut args = A::query_arguments();
        args.extend(B::query_arguments());
        args
    }

    fn build_query_ptr(columns: &[Option<*mut Column>], cursor: &mut usize) -> Self::QueryPtr {
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

impl<'w, A, B, C> QueryParameter<'w> for (A, B, C)
where
    A: QueryParameter<'w>,
    B: QueryParameter<'w>,
    C: QueryParameter<'w>,
{
    type Item = (A::Item, B::Item, C::Item);
    type QueryPtr = (A::QueryPtr, B::QueryPtr, C::QueryPtr);

    fn query_arguments() -> Vec<QueryArgument> {
        let mut args = A::query_arguments();
        args.extend(B::query_arguments());
        args.extend(C::query_arguments());
        args
    }

    fn build_query_ptr(columns: &[Option<*mut Column>], cursor: &mut usize) -> Self::QueryPtr {
        let a = A::build_query_ptr(columns, cursor);
        let b = B::build_query_ptr(columns, cursor);
        let c = C::build_query_ptr(columns, cursor);
        (a, b, c)
    }

    unsafe fn fetch(query_ptr: &Self::QueryPtr, index: usize) -> Option<Self::Item> {
        let a = unsafe { A::fetch(&query_ptr.0, index) };
        let b = unsafe { B::fetch(&query_ptr.1, index) };
        let c = unsafe { C::fetch(&query_ptr.2, index) };

        a.and_then(|a| b.and_then(|b| c.map(|c| (a, b, c))))
    }
}

pub struct Query<'w, T: QueryParameter<'w>> {
    pub world: &'w mut World,
    // Query depends on T, but doesn't actually contain a reference to it.
    // So this is here to keep the compiler happy.
    _marker: PhantomData<T>,
}

impl<'a, T: QueryParameter<'a>> Query<'a, T> {
    fn new(world: &'a mut World) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }
}

impl<T: for<'w> QueryParameter<'w> + 'static> SystemParameter for Query<'_, T> {
    type Item<'w, 's, 'e> = Query<'w, T>;

    fn get<'w, 's, 'e>(
        world: &'w mut World,
        _state: &'s mut SystemState,
        _access: &'e mut dyn ApplicationAccess,
    ) -> Self::Item<'w, 's, 'e> {
        Query::new(world)
    }
}

impl<'w, T: QueryParameter<'w>> Query<'w, T> {
    // TODO make this unsafe so we dont have to make queries mutable
    pub fn iter(&mut self) -> impl Iterator<Item = T::Item> {
        let query_arguments = T::query_arguments();

        let archetype_columns = self.world.find_matching_archetype_columns(&query_arguments);

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

pub struct QueryIterator<'w, T: QueryParameter<'w>> {
    component_index: usize,
    matching_archetypes: Vec<T::QueryPtr>,
    archetype_index: usize,
}

impl<'q, 'w, T> Iterator for QueryIterator<'w, T>
where
    T: QueryParameter<'w>,
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
