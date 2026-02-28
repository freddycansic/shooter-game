use crate::ecs::archetype::Archetype;
use fxhash::{FxHashMap, FxHasher};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::hash::Hasher;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct StableComponentId(pub [u8; 20]);

impl StableComponentId {
    pub const fn from_str(string: &str) -> Self {
        Self(const_sha1::sha1(string.as_bytes()).as_bytes())
    }
}

pub trait Component {
    const ID: StableComponentId;
}

pub trait Components {
    fn ids() -> Vec<StableComponentId>;
    fn spawn(self, archetype: &mut Archetype);
    fn archetype_id() -> u64 {
        let mut hasher = FxHasher::default();
        for component_id in Self::ids() {
            for block in component_id.0.iter() {
                hasher.write_u8(*block);
            }
        }
        hasher.finish()
    }
}

impl<A: Component + 'static> Components for A {
    fn ids() -> Vec<StableComponentId> {
        vec![A::ID]
    }

    fn spawn(self, archetype: &mut Archetype) {
        archetype.columns[0].as_type_mut_unchecked::<A>().push(self);
    }
}

impl<A: Component + 'static, B: Component + 'static> Components for (A, B) {
    fn ids() -> Vec<StableComponentId> {
        let mut ids = vec![A::ID, B::ID];
        ids.sort_unstable();
        ids
    }

    fn spawn(self, archetype: &mut Archetype) {
        let (a, b) = self;

        archetype.columns[0].as_type_mut_unchecked::<A>().push(a);
        archetype.columns[1].as_type_mut_unchecked::<B>().push(b);
    }
}

pub struct ComponentRegistry {
    components: FxHashMap<TypeId, StableComponentId>,
}

impl ComponentRegistry {
    pub fn register<T: Component + 'static>(&mut self) {
        self.components.insert(TypeId::of::<T>(), T::ID);
    }
}
