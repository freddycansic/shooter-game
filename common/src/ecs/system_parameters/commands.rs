use crate::executor::CommandExecutor;
use common::ecs::system::SystemState;
use common::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use std::ops::{Deref, DerefMut};

pub struct Commands<'e>(&'e mut (dyn CommandExecutor + 'e));

impl<'e> Deref for Commands<'e> {
    type Target = dyn CommandExecutor + 'e;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'e, 'r> DerefMut for Commands<'e> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'r> SystemParameter for Commands<'_> {
    type Item<'w, 's, 'e> = Commands<'e>;

    fn get<'w, 's, 'e>(
        world: &'w mut World,
        state: &'s mut SystemState,
        executor: &'e mut dyn CommandExecutor,
    ) -> Self::Item<'w, 's, 'e> {
        Commands(executor)
    }
}
