use crate::ecs::event::{Event, EventQueue};
use crate::ecs::system_parameters::system_parameter::SystemParameter;
use common::world::World;
use std::marker::PhantomData;

pub struct EventReader<'w, T: Event> {
    queue: &'w EventQueue,
    last_index: usize,
    _marker: PhantomData<T>,
}

impl<'w, T: Event + 'static> EventReader<'w, T> {
    pub fn new(queue: &'w EventQueue) -> Self {
        Self {
            queue,
            last_index: queue.0.len(),
            _marker: PhantomData,
        }
    }

    pub fn read(&mut self) -> impl Iterator<Item = &T> {
        // only read new events
        dbg!(self.last_index);
        let events = &self.queue.0[self.last_index..];

        self.last_index = self.queue.0.len();

        dbg!(self.last_index);

        events
            .iter()
            .map(|event| event.as_any_ref().downcast_ref::<T>().unwrap())
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
