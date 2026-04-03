use crate::ecs::archetype::{Archetype, Column};
use fxhash::{FxHashMap, FxHasher};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::hash::Hasher;
pub(crate) use crate::ecs::stable_id::StableId;

pub fn archetype_id(ids: &[StableId]) -> u64 {
    let mut hasher = FxHasher::default();
    for component_id in ids.iter() {
        for block in component_id.0.iter() {
            hasher.write_u8(*block);
        }
    }
    hasher.finish()
}

pub trait Component {
    const ID: StableId;
}

pub struct ComponentRegistry {
    components: FxHashMap<TypeId, StableId>,
}

impl ComponentRegistry {
    pub fn register<T: Component + 'static>(&mut self) {
        self.components.insert(TypeId::of::<T>(), T::ID);
    }
}
