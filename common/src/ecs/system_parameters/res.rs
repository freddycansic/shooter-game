use std::ops::{Deref, DerefMut};
use crate::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use crate::ecs::resource::Resource;

pub struct Res<'w, T: Resource>(&'w T);

impl<'w, T: Resource> Deref for Res<'w, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: Resource + 'static> SystemParameter for Res<'_, T> {
    type Item<'w> = Res<'w, T>;

    fn get(world: &mut World) -> Self::Item<'_> {
        Res(world.resource::<T>().unwrap())
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
    type Item<'w> = ResMut<'w, T>;

    fn get(world: &mut World) -> Self::Item<'_> {
        ResMut(world.resource_mut::<T>().unwrap())
    }
}
