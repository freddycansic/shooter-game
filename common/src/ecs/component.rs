pub(crate) use crate::ecs::stable_id::StableId;
use fxhash::{FxHashMap, FxHasher};
use std::any::TypeId;
use std::hash::Hasher;

pub fn archetype_id(ids: &[StableId]) -> u64 {
    let mut hasher = FxHasher::default();
    for component_id in ids.iter() {
        hasher.write_u64(component_id.0);
    }
    hasher.finish()
}

pub trait Component {
    const ID: StableId;
    const NAME: &'static str;
}

// TODO ununsed?
pub struct ComponentRegistry {
    components: FxHashMap<TypeId, StableId>,
}

impl ComponentRegistry {
    pub fn register<T: Component + 'static>(&mut self) {
        self.components.insert(TypeId::of::<T>(), T::ID);
    }
}
