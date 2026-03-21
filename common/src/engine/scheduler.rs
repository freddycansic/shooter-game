use crate::ecs::system::{System, IntoSystem};
use crate::world::World;

pub struct Scheduler {
    pub systems: Vec<System>,
}

impl Scheduler {
    pub fn register<S, P>(&mut self, system: S)
    where
        S: IntoSystem<P>,
    {
        self.systems.push(system.into_system());
    }

    pub fn run_systems(&mut self, world: &mut World) {
        for system in self.systems.iter_mut() {
            system.run(world);
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            systems: vec![]
        }
    }
}