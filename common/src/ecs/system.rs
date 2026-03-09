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

    fn run(&mut self, world: &mut World) {
        (self.function)(world);
    }
}

pub trait SystemParameter<'w>: Sized {
    fn get(world: &'w mut World) -> Self;
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

impl<'w, T: Components + 'static> SystemParameter<'w> for Query<'w, T> {
    fn get(world: &'w mut World) -> Self {
        Query::new(vec![world.find_archetype::<T>()])
    }
}

impl<'a, F, P1> IntoSystem<(P1,)> for F
where
    // for syntax is basically saying: this function works with any lifetime
    // Without it, a fixed lifetime would be given to the closure, meaning that it would not be
    // possible to call the function many times with different World lifetimes.
    F: for<'w> Fn(P1) + 'static,
    P1: for<'w> SystemParameter<'w>,
{
    fn into_system(self) -> System {
        System::new(move |world: &mut World| {
            let p1 = P1::get(world);
            self(p1);
        })
    }
}

impl<'a, F, P1, P2> IntoSystem<(P1, P2)> for F
where
    F: for<'w> Fn(P1, P2) + 'static,
    P1: for<'w> SystemParameter<'w>,
    P2: for<'w> SystemParameter<'w>,
{
    fn into_system(self) -> System {
        System::new(move |world: &mut World| {
            let p1 = P1::get(world);
            let p2 = P2::get(world);
            self(p1, p2);
        })
    }
}

pub trait IntoSystem<P> {
    fn into_system(self) -> System;
}

/// A container which holds registered systems
pub struct Systems {
    systems: Vec<System>,
}

impl Systems {
    pub fn register<S, P>(&mut self, system: S)
    where
        S: IntoSystem<P>,
    {
        self.systems.push(system.into_system());
    }

    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }
}

impl Default for Systems {
    fn default() -> Self {
        Self {
            systems: Vec::new(),
        }
    }
}