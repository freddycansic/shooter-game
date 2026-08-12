use crate::runtime::ApplicationAccess;
use common::ecs::system::SystemState;
use common::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use std::ops::{Deref, DerefMut};

pub struct ApplicationContext<'e>(&'e mut (dyn ApplicationAccess + 'e));

impl<'e> Deref for ApplicationContext<'e> {
    type Target = dyn ApplicationAccess + 'e;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'e, 'r> DerefMut for ApplicationContext<'e> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'r> SystemParameter for ApplicationContext<'_> {
    type Item<'w, 's, 'e> = ApplicationContext<'e>;

    fn get<'w, 's, 'e>(
        _world: &'w mut World,
        _state: &'s mut SystemState,
        implementation: &'e mut dyn ApplicationAccess,
    ) -> Self::Item<'w, 's, 'e> {
        ApplicationContext(implementation)
    }
}
