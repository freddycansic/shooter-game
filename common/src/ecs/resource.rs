use common::ecs::component::StableId;
use std::any::Any;

pub trait Resource {
    const ID: StableId;
}

// TODO make AsAny trait cause this is duplicated
trait ResourceData {
    fn as_any_ref(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Resource + 'static> ResourceData for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct ResourceStore {
    resource: Box<dyn ResourceData>,
}

impl<T: Resource + 'static> From<T> for ResourceStore {
    fn from(value: T) -> Self {
        Self {
            resource: Box::new(value),
        }
    }
}

impl ResourceStore {
    pub fn get<T: Resource + 'static>(&self) -> Option<&T> {
        self.resource.as_any_ref().downcast_ref::<T>()
    }

    pub fn get_mut<T: Resource + 'static>(&mut self) -> Option<&mut T> {
        self.resource.as_any_mut().downcast_mut::<T>()
    }
}
