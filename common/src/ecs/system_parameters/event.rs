use std::marker::PhantomData;
use crate::ecs::event::{Event, EventQueue};
use crate::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;

pub struct EventReader<'w, T: Event> {
    queue: &'w mut EventQueue,
    _marker: PhantomData<T>,
}

impl<'w, T: Event + 'static> EventReader<'w, T> {
    pub fn new(queue: &'w mut EventQueue) -> Self {
        Self {
            queue,
            _marker: PhantomData,
        }
    }
    
    pub fn drain(&mut self) -> impl Iterator<Item = T> {
        self.queue.drain().map(|event| *event.into_any().downcast::<T>().unwrap())
    }
}

impl<T: Event + 'static> SystemParameter for EventReader<'_, T> {
    type Item<'w> = EventReader<'w, T>;

    fn get(world: &mut World) -> Self::Item<'_> {
        EventReader::new(world.event_queue::<T>())
    }
}

pub struct EventWriter<'w, T: Event> {
    queue: &'w mut EventQueue,
    _marker: PhantomData<T>,
}

impl<'w, T: Event + 'static> EventWriter<'w, T> {
    pub fn new(queue: &'w mut EventQueue) -> Self {
        Self {
            queue,
            _marker: PhantomData,
        }
    }
    
    pub fn send(&mut self, event: T) {
        self.queue.send(Box::new(event));
    }
}

impl<T: Event + 'static> SystemParameter for EventWriter<'_, T> {
    type Item<'w> = EventWriter<'w, T>;

    fn get(world: &mut World) -> Self::Item<'_> {
        EventWriter::new(world.event_queue::<T>())
    }
}
