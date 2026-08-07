use common::engine::scheduler::Scheduler;
use common::world::World;

pub trait Subsystem {
    fn register_resources(world: &mut World);
    fn register_systems(scheduler: &mut Scheduler);
}
