use std::hash::Hasher;
use fxhash::FxHasher;
use crate::ecs::component::Component;
use common::ecs::archetype::Archetype;
use common::ecs::component::StableComponentId;

// These are components which are owned and can be consumed by an archetype
pub trait OwnedComponents {
    fn ids() -> Vec<StableComponentId>;
    fn spawn(self, archetype: &mut Archetype);
}

impl<A: Component + 'static> OwnedComponents for A {
    fn ids() -> Vec<StableComponentId> { vec![A::ID] }
    
    fn spawn(self, archetype: &mut Archetype) {
        archetype.columns[0].as_type_mut_unchecked::<A>().push(self);
    }
}

impl<A: Component + 'static, B: Component + 'static> OwnedComponents for (A, B) {
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
