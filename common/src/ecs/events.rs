use crate::ecs::component::StableId;
use std::any::Any;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub trait Event: Send + 'static {
    const ID: StableId;
    const NAME: &'static str;
}

// TODO make this a type erased Vec<T> like column for cache locality.
// I forgot you could do this
pub trait EventMessage: Send + 'static {
    fn as_any_ref(&self) -> &dyn Any;
}

impl<T: Event + 'static> EventMessage for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

pub struct Events {
    pub queue: Vec<Box<dyn EventMessage>>,
    pub sender: Sender<Box<dyn EventMessage>>,
    pub receiver: Receiver<Box<dyn EventMessage>>,
}

impl Default for Events {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            queue: vec![],
            sender,
            receiver,
        }
    }
}

impl Events {
    pub fn write(&mut self, event: Box<dyn EventMessage>) {
        self.queue.push(event);
    }

    /// Read all the incoming events which have come from another thread.
    /// Push on to normal queue, ready to be used.
    pub fn consume_external(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            self.queue.push(event);
        }
    }
}
