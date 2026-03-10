use crate::ecs::archetype::Archetype;
use crate::ecs::component::Components;
use crate::world::World;
use std::marker::PhantomData;

pub struct System {
    function: Box<dyn FnMut(&mut World)>,
}

impl System {
    fn new<T: FnMut(&mut World) + 'static>(function: T) -> Self {
        Self {
            function: Box::new(function),
        }
    }

    pub fn run(&mut self, world: &mut World) {
        (self.function)(world);
    }
}

pub trait SystemParameter: Sized {
    type Item<'w>: SystemParameter;
    fn get(world: &mut World) -> Self::Item<'_>;
}

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

impl<F, P1> IntoSystem<(P1,)> for F
where
    // for syntax is basically saying: this function works with any lifetime
    // Without it, a fixed lifetime would be given to the closure at compile time, meaning that it
    // would not be possible to call the function many times with different World lifetimes.
    F: for<'w> Fn(<P1 as SystemParameter>::Item<'w>) + 'static,
    P1: SystemParameter,
{
    fn into_system(self) -> System {
        System::new(move |world: &mut World| {
            let p1 = P1::get(world);
            self(p1);
        })
    }
}

impl<F, P1, P2> IntoSystem<(P1, P2)> for F
where
    F: for<'w> Fn(<P1 as SystemParameter>::Item<'w>, <P2 as SystemParameter>::Item<'w>) + 'static,
    P1: SystemParameter,
    P2: SystemParameter,
{
    fn into_system(self) -> System {
        System::new(move |world: &mut World| {
            // TODO validate that P1 != P2, so that they are not accessing the same memory
            let world_ptr = world as *mut World;
            let p1 = P1::get(unsafe { &mut *world_ptr });
            let p2 = P2::get(unsafe { &mut *world_ptr });
            self(p1, p2);
        })
    }
}

pub trait IntoSystem<P> {
    fn into_system(self) -> System;
}