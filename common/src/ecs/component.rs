use crate::ecs::archetype::Archetype;
use fxhash::FxHashMap;
use std::any::TypeId;

#[derive(PartialEq, PartialOrd, Ord, Eq)]
pub struct StableComponentId(pub u64);

pub trait Component {
    const ID: StableComponentId;
}

pub trait Components {
    fn ids() -> Vec<StableComponentId>;
    fn spawn(self, archetype: &mut Archetype);
    fn archetype_id() -> u32 {
        Self::ids().iter().fold(0, |acc, id| acc | id.0)
    }
}

impl<A: Component + 'static> Components for A {
    fn ids() -> Vec<StableComponentId> {
        vec![A::ID]
    }

    fn spawn(self, archetype: &mut Archetype) {
        archetype.columns[0].as_type_mut::<A>().unwrap().push(self);
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

        archetype.columns[0].as_type_mut::<A>().unwrap().push(a);
        archetype.columns[1].as_type_mut::<B>().unwrap().push(b);
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
