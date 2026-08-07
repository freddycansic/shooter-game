use crate::ecs::event::{Event, EventQueue};
use crate::ecs::system_parameters::system_parameter::SystemParameter;
use crate::executor::CommandExecutor;
use common::ecs::system::SystemState;
use common::world::World;
use std::marker::PhantomData;

pub struct EventReader<'w, 's, T: Event> {
    queue: &'w EventQueue,
    event_cursor: &'s mut usize,
    _marker: PhantomData<T>,
}

impl<'w, 's, T: Event + 'static> EventReader<'w, 's, T> {
    pub fn new(queue: &'w EventQueue, event_cursor: &'s mut usize) -> Self {
        Self {
            queue,
            event_cursor,
            _marker: PhantomData,
        }
    }

    pub fn read(&mut self) -> impl Iterator<Item = &T> {
        // only read new events
        let events = &self.queue.0[*self.event_cursor..];

        *self.event_cursor = self.queue.0.len();

        events
            .iter()
            .map(|event| event.as_any_ref().downcast_ref::<T>().unwrap())
    }
}

impl<T: Event + 'static> SystemParameter for EventReader<'_, '_, T> {
    type Item<'w, 's, 'e> = EventReader<'w, 's, T>;

    fn get<'w, 's, 'e>(
        world: &'w mut World,
        state: &'s mut SystemState,
        executor: &'e mut dyn CommandExecutor,
    ) -> Self::Item<'w, 's, 'e> {
        EventReader::new(
            world.event_queue::<T>(),
            state.event_reader_cursors.entry(T::ID).or_insert(0),
        )
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

    pub fn write(&mut self, event: T) {
        self.queue.write(Box::new(event));
    }
}

impl<T: Event + 'static> SystemParameter for EventWriter<'_, T> {
    type Item<'w, 's, 'e> = EventWriter<'w, T>;

    fn get<'w, 's, 'e>(
        world: &'w mut World,
        state: &'s mut SystemState,
        executor: &'e mut dyn CommandExecutor,
    ) -> Self::Item<'w, 's, 'e> {
        EventWriter::new(world.event_queue::<T>())
    }
}
