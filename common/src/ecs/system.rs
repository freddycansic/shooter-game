use fxhash::FxHashMap;
use crate::ecs::component::StableId;
use crate::ecs::system_parameters::system_parameter::SystemParameter;
use crate::world::World;

pub struct System {
    function: Box<dyn FnMut(&mut World)>,
    event_reader_cursors: FxHashMap<StableId, Vec<usize>>
}

impl System {
    fn new<T: FnMut(&mut World) + 'static>(function: T) -> Self {
        Self {
            function: Box::new(function),
            event_reader_cursors: FxHashMap::default()
        }
    }

    pub fn run(&mut self, world: &mut World) {
        (self.function)(world);
    }
}

impl<F, P1> IntoSystem<(P1,)> for F
where
    // for syntax is basically saying: this function works with any lifetime
    // Without it, a fixed lifetime would be given to the closure at compile time, meaning that it
    // would not be possible to call the function many times with different World lifetimes.
    F: Fn(P1) + for<'w> Fn(<P1 as SystemParameter>::Item<'w>) + 'static,
    P1: SystemParameter,
{
    fn into_system(self) -> System {
        System::new(move |world: &mut World| {
            let p1 = P1::get(world);

            fn call_inner<A, FInner: Fn(A)>(f: &FInner, a: A) {
                f(a);
            }

            call_inner(&self, p1);
        })
    }
}

impl<F, P1, P2> IntoSystem<(P1, P2)> for F
where
    F: Fn(P1, P2) + for<'w> Fn(<P1 as SystemParameter>::Item<'w>, <P2 as SystemParameter>::Item<'w>) + 'static,
    P1: SystemParameter,
    P2: SystemParameter,
{
    fn into_system(self) -> System {
        System::new(move |world: &mut World| {
            let world_ptr = world as *mut World;
            // TODO validate that P1 != P2, so that they are not accessing the same memory
            let p1 = P1::get(unsafe { &mut *world_ptr });
            let p2 = P2::get(unsafe { &mut *world_ptr });

            fn call_inner<A, B, FInner: Fn(A, B)>(f: &FInner, a: A, b: B) {
                f(a, b);
            }

            call_inner(&self, p1, p2);
        })
    }
}

pub trait IntoSystem<P> {
    fn into_system(self) -> System;
}
