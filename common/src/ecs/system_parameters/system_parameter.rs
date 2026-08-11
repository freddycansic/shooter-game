use crate::executor::CommandExecutor;
use common::ecs::system::SystemState;
use common::world::World;

pub trait SystemParameter: Sized {
    type Item<'w, 's, 'e>: SystemParameter;
    fn get<'w, 's, 'e>(
        world: &'w mut World,
        state: &'s mut SystemState,
        executor: &'e mut dyn CommandExecutor,
    ) -> Self::Item<'w, 's, 'e>;
}
