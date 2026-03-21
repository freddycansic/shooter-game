use common::ecs::archetype::Archetype;
use common::ecs::component::Components;
use common::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use std::marker::PhantomData;

pub struct Query<'a, T: Components> {
    pub archetypes: Vec<&'a mut Archetype>,
    // Query depends on T, but doesn't actually contain a reference to it.
    // So this is here to keep the compiler happy.
    _marker: PhantomData<T>,
}

impl<'a, T: Components> Query<'a, T> {
    fn new(archetypes: Vec<&'a mut Archetype>) -> Self {
        Self {
            archetypes,
            _marker: PhantomData,
        }
    }
}

impl<T: Components + 'static> SystemParameter for Query<'_, T> {
    type Item<'w> = Query<'w, T>;

    fn get(world: &mut World) -> Self::Item<'_> {
        Query::new(vec![world.find_archetype::<T>()])
    }
}
