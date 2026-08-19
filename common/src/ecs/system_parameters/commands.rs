use crate::ecs::entity::Entity;
use crate::ecs::owned_components::OwnedComponents;
use common::ecs::system::SystemState;
use common::ecs::system_parameters::system_parameter::SystemParameter;
use common::runtime::ApplicationAccess;
use common::world::World;

pub struct Commands<'w>(&'w mut World);

impl<'w> Commands<'w> {
    pub fn spawn<T: OwnedComponents + 'static>(&mut self, components: T) {
        self.0.command_queue.queue_spawn(components);
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.0.command_queue.queue_destroy(entity);
    }
}

impl SystemParameter for Commands<'_> {
    type Item<'w, 's, 'e> = Commands<'w>;

    fn get<'w, 's, 'e>(
        world: &'w mut World,
        _state: &'s mut SystemState,
        _access: &'e mut dyn ApplicationAccess,
    ) -> Self::Item<'w, 's, 'e> {
        Commands(world)
    }
}
