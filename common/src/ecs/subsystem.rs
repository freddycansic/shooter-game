use crate::runtime::RuntimeContext;
use common::engine::scheduler::Scheduler;
use common::world::World;

pub trait Subsystem {
    fn register_resources(world: &mut World, context: Option<&RuntimeContext>);
    fn register_systems(scheduler: &mut Scheduler);
}
