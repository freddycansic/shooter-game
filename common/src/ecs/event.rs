use crate::ecs::component::StableId;
use std::any::Any;
use std::collections::VecDeque;

pub trait Event {
    const ID: StableId;
}

pub trait EventMessage {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Event + 'static> EventMessage for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

#[derive(Default)]
pub struct EventQueue(VecDeque<Box<dyn EventMessage>>);

impl EventQueue {
    pub fn send(&mut self, event: Box<dyn EventMessage>) {
        self.0.push_back(event);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Box<dyn EventMessage>> {
        self.0.drain(..)
    }
}
