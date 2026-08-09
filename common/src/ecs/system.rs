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

pub trait IntoSystem<P> {
    fn into_system(self) -> System;
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

macro_rules! impl_into_system {
    ($($P:ident),+) => {
        impl<F, $($P),+> IntoSystem<($($P),+)> for F
        where
            F: Fn($($P),+)
                + for<'w, 's, 'e> Fn(
                    $($P::Item<'w, 's, 'e>),+
                )
                + 'static,
            $($P: SystemParameter),+
        {
            fn into_system(self) -> System {
                System::new(
                    move |
                        world: &mut World,
                        state: &mut SystemState,
                        executor: &mut dyn CommandExecutor
                    | {
                        let world_ptr = world as *mut World;
                        let state_ptr = state as *mut SystemState;
                        let executor_ptr = executor as *mut dyn CommandExecutor;

                        $(
                            let $P = $P::get(
                                unsafe { &mut *world_ptr },
                                unsafe { &mut *state_ptr },
                                unsafe { &mut *executor_ptr },
                            );
                        )+

                        self($($P),+);
                    },
                )
            }
        }
    };
}

impl_into_system!(P1, P2);
impl_into_system!(P1, P2, P3);
impl_into_system!(P1, P2, P3, P4);
impl_into_system!(P1, P2, P3, P4, P5);
impl_into_system!(P1, P2, P3, P4, P5, P6);
impl_into_system!(P1, P2, P3, P4, P5, P6, P7);
impl_into_system!(P1, P2, P3, P4, P5, P6, P7, P8);
impl_into_system!(P1, P2, P3, P4, P5, P6, P7, P8, P9);
