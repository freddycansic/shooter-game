use crate::ecs::component::Component;
use crate::ecs::stable_id::StableId;
use crate::world::World;
use common::ecs::entity::Entity;

// These are components which are owned and can be consumed by an archetype
pub trait OwnedComponents {
    fn sorted_ids() -> Vec<StableId>
    where
        Self: Sized;

    // This function should only be performed on boxed versions of itself.
    // Cause they need to be trait objects when spawning.
    fn spawn(self: Box<Self>, world: &mut World) -> Entity;
}

impl<A: Component + 'static> OwnedComponents for A {
    fn sorted_ids() -> Vec<StableId> {
        vec![A::ID]
    }

    fn spawn(self: Box<Self>, world: &mut World) -> Entity {
        let archetype = world.find_exact_archetype(&Self::sorted_ids());
        archetype.columns[0].as_type_mut_unchecked::<A>().push(*self);

        let entity = Entity {
            archetype_id: archetype.id,
            row: archetype.entities.len() as u32,
        };

        archetype.entities.push(entity.clone());

        entity
    }
}

macro_rules! impl_owned_components {
    ($($T:ident),+) => {
        impl<$($T: Component + 'static),+> OwnedComponents for ($($T,)+) {
            fn sorted_ids() -> Vec<StableId> {
                let mut ids = vec![$($T::ID),+];
                ids.sort_unstable();
                ids
            }

            fn spawn(self: Box<Self>, world: &mut World) -> Entity {
                let archetype = world.find_exact_archetype(&Self::sorted_ids());

                let ($($T,)+) = *self;

                $(
                    archetype
                        .column_for_id_mut::<$T>()
                        .unwrap()
                        .as_type_mut_unchecked::<$T>()
                        .push($T);
                )+

                let entity = Entity {
                    archetype_id: archetype.id,
                    row: archetype.entities.len() as u32,
                };

                archetype.entities.push(entity.clone());

                entity
            }
        }
    };
}

impl_owned_components!(A, B);
impl_owned_components!(A, B, C);
impl_owned_components!(A, B, C, D);
impl_owned_components!(A, B, C, D, E);
