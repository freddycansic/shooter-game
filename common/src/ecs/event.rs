use crate::ecs::component::StableId;
use std::any::Any;

pub trait Event {
    const ID: StableId;
    const NAME: &'static str;
}

// TODO make this a type erased Vec<T> like column for cache locality.
// I forgot you could do this
pub trait EventMessage {
    fn as_any_ref(&self) -> &dyn Any;
}

impl<T: Event + 'static> EventMessage for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
pub struct EventQueue(pub Vec<Box<dyn EventMessage>>);

impl EventQueue {
    pub fn write(&mut self, event: Box<dyn EventMessage>) {
        self.0.push(event);
    }
}
