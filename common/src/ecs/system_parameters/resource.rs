use crate::ecs::system_parameters::system_parameter::SystemParameter;
use common::ecs::component::StableId;
use common::world::World;

pub trait Resource {
    const ID: StableId;
}

pub struct Res<'w, T: Resource>(&'w T);

impl<T: Resource + 'static> SystemParameter for Res<'_, T> {
    type Item<'w> = Res<'w, T>;

    fn get(world: &mut World) -> Self::Item<'_> {
        // world.get_resource::<T>()
        todo!()
    }
}

pub struct ResMut<'w, T: Resource>(&'w mut T);

impl<T: Resource + 'static> SystemParameter for ResMut<'_, T> {
    type Item<'w> = ResMut<'w, T>;

    fn get(world: &mut World) -> Self::Item<'_> {
        todo!()
    }
}
