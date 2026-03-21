use common::world::World;

pub trait SystemParameter: Sized {
    type Item<'w>: SystemParameter;
    fn get(world: &mut World) -> Self::Item<'_>;
}
