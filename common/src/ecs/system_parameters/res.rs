use crate::ecs::resource::Resource;
use crate::ecs::system_parameters::system_parameter::SystemParameter;
use crate::executor::CommandExecutor;
use common::ecs::system::SystemState;
use common::world::World;
use egui_glium::egui_winit::egui::TextBuffer;
use std::ops::{Deref, DerefMut};

pub struct Res<'w, T: Resource>(&'w T);

impl<'w, T: Resource> Deref for Res<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: Resource + 'static> SystemParameter for Res<'_, T> {
    type Item<'w, 's, 'e> = Res<'w, T>;

    fn get<'w, 's, 'e>(
        world: &'w mut World,
        _state: &'s mut SystemState,
        _executor: &'e mut dyn CommandExecutor,
    ) -> Self::Item<'w, 's, 'e> {
        Res(world
            .resource::<T>()
            .expect(format!("The resource {} has not been registered with the world.", T::NAME).as_str()))
    }
}

pub struct ResMut<'w, T: Resource>(&'w mut T);

impl<'w, T: Resource> Deref for ResMut<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'w, T: Resource> DerefMut for ResMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<T: Resource + 'static> SystemParameter for ResMut<'_, T> {
    type Item<'w, 's, 'e> = ResMut<'w, T>;

    fn get<'w, 's, 'e>(
        world: &'w mut World,
        _state: &'s mut SystemState,
        _executor: &'e mut dyn CommandExecutor,
    ) -> Self::Item<'w, 's, 'e> {
        ResMut(
            world
                .resource_mut::<T>()
                .expect(format!("The resource {} has not been registered with the world.", T::NAME).as_str()),
        )
    }
}
