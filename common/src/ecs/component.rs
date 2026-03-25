use crate::ecs::archetype::{Archetype, Column};
use fxhash::{FxHashMap, FxHasher};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::hash::Hasher;

pub fn archetype_id(ids: &[StableComponentId]) -> u64 {
    let mut hasher = FxHasher::default();
    for component_id in ids.iter() {
        for block in component_id.0.iter() {
            hasher.write_u8(*block);
        }
    }
    hasher.finish()
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Serialize, Deserialize)]
pub struct StableComponentId(pub [u8; 20]);

impl StableComponentId {
    pub const fn from_str(string: &str) -> Self {
        Self(const_sha1::sha1(string.as_bytes()).as_bytes())
    }
}

pub trait Component {
    const ID: StableComponentId;
}

pub struct ComponentRegistry {
    components: FxHashMap<TypeId, StableComponentId>,
}

impl ComponentRegistry {
    pub fn register<T: Component + 'static>(&mut self) {
        self.components.insert(TypeId::of::<T>(), T::ID);
    }
}
