use crate::ecs::component::StableId;
use crate::ecs::system_parameters::system_parameter::SystemParameter;
use crate::executor::CommandExecutor;
use crate::world::World;
use derive_more::Debug;
use fxhash::FxHashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct System {
    pub id: StableId,
    #[debug("traitname")] // trait objects don't implement std::fmt::Debug by default
    pub function: Box<dyn FnMut(&mut World, &mut SystemState, &mut dyn CommandExecutor)>,
    pub state: SystemState,
}

#[derive(Debug)]
pub struct SystemState {
    pub event_reader_cursors: FxHashMap<StableId, usize>,
    pub trigger_cursors: FxHashMap<StableId, usize>,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            event_reader_cursors: FxHashMap::default(),
            trigger_cursors: FxHashMap::default(),
        }
    }
}

impl System {
    fn new<T: FnMut(&mut World, &mut SystemState, &mut dyn CommandExecutor) + 'static>(function: T) -> Self {
        Self {
            id: StableId::from_str(std::any::type_name::<T>()),
            function: Box::new(function),
            state: SystemState::default(),
        }
    }

    pub fn run(&mut self, world: &mut World, executor: &mut dyn CommandExecutor) {
        (self.function)(world, &mut self.state, executor);
    }
}

impl PartialEq for System {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for System {}

impl Hash for System {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<F> IntoSystem<()> for F
where
    F: Fn() + 'static,
{
    fn into_system(self) -> System {
        System::new(move |_: &mut World, _: &mut SystemState, _: &mut dyn CommandExecutor| {
            self();
        })
    }
}

impl<F, P1> IntoSystem<(P1,)> for F
where
    // for syntax is basically saying: this function works with any lifetime
    // Without it, a fixed lifetime would be given to the closure at compile time, meaning that it
    // would not be possible to call the function many times with different World lifetimes.
    F: Fn(P1) + for<'w, 's, 'e> Fn(<P1 as SystemParameter>::Item<'w, 's, 'e>) + 'static,
    P1: SystemParameter,
{
    fn into_system(self) -> System {
        System::new(
            move |world: &mut World, state: &mut SystemState, executor: &mut dyn CommandExecutor| {
                let p1 = P1::get(world, state, executor);

                fn call_inner<A, FInner: Fn(A)>(f: &FInner, a: A) {
                    f(a);
                }

                call_inner(&self, p1);
            },
        )
    }
}

impl<F, P1, P2> IntoSystem<(P1, P2)> for F
where
    F: Fn(P1, P2)
        + for<'w, 's, 'e> Fn(<P1 as SystemParameter>::Item<'w, 's, 'e>, <P2 as SystemParameter>::Item<'w, 's, 'e>)
        + 'static,
    P1: SystemParameter,
    P2: SystemParameter,
{
    fn into_system(self) -> System {
        System::new(
            move |world: &mut World, state: &mut SystemState, executor: &mut dyn CommandExecutor| {
                let world_ptr = world as *mut World;
                let state_ptr = state as *mut SystemState;
                let executor_ptr = executor as *mut dyn CommandExecutor;
                // TODO validate that P1 != P2, so that they are not accessing the same memory
                let p1 = P1::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });
                let p2 = P2::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });

                fn call_inner<A, B, FInner: Fn(A, B)>(f: &FInner, a: A, b: B) {
                    f(a, b);
                }

                call_inner(&self, p1, p2);
            },
        )
    }
}

impl<F, P1, P2, P3> IntoSystem<(P1, P2, P3)> for F
where
    F: Fn(P1, P2, P3)
        + for<'w, 's, 'e> Fn(
            <P1 as SystemParameter>::Item<'w, 's, 'e>,
            <P2 as SystemParameter>::Item<'w, 's, 'e>,
            <P3 as SystemParameter>::Item<'w, 's, 'e>,
        ) + 'static,
    P1: SystemParameter,
    P2: SystemParameter,
    P3: SystemParameter,
{
    fn into_system(self) -> System {
        System::new(
            move |world: &mut World, state: &mut SystemState, executor: &mut dyn CommandExecutor| {
                let world_ptr = world as *mut World;
                let state_ptr = state as *mut SystemState;
                let executor_ptr = executor as *mut dyn CommandExecutor;
                // TODO validate that P1 != P2 != P3, so that they are not accessing the same memory
                let p1 = P1::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });
                let p2 = P2::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });
                let p3 = P3::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });

                fn call_inner<A, B, C, FInner: Fn(A, B, C)>(f: &FInner, a: A, b: B, c: C) {
                    f(a, b, c);
                }

                call_inner(&self, p1, p2, p3);
            },
        )
    }
}

impl<F, P1, P2, P3, P4> IntoSystem<(P1, P2, P3, P4)> for F
where
    F: Fn(P1, P2, P3, P4)
        + for<'w, 's, 'e> Fn(
            <P1 as SystemParameter>::Item<'w, 's, 'e>,
            <P2 as SystemParameter>::Item<'w, 's, 'e>,
            <P3 as SystemParameter>::Item<'w, 's, 'e>,
            <P4 as SystemParameter>::Item<'w, 's, 'e>,
        ) + 'static,
    P1: SystemParameter,
    P2: SystemParameter,
    P3: SystemParameter,
    P4: SystemParameter,
{
    fn into_system(self) -> System {
        System::new(
            move |world: &mut World, state: &mut SystemState, executor: &mut dyn CommandExecutor| {
                let world_ptr = world as *mut World;
                let state_ptr = state as *mut SystemState;
                let executor_ptr = executor as *mut dyn CommandExecutor;
                // TODO validate that P1 != P2 != P3 != P4, so that they are not accessing the same memory
                let p1 = P1::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });
                let p2 = P2::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });
                let p3 = P3::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });
                let p4 = P4::get(unsafe { &mut *world_ptr }, unsafe { &mut *state_ptr }, unsafe {
                    &mut *executor_ptr
                });

                fn call_inner<A, B, C, D, FInner: Fn(A, B, C, D)>(f: &FInner, a: A, b: B, c: C, d: D) {
                    f(a, b, c, d);
                }

                call_inner(&self, p1, p2, p3, p4);
            },
        )
    }
}

pub trait IntoSystem<P> {
    fn into_system(self) -> System;
}
