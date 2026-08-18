use crate::ecs::events::{Event, EventMessage, Events};
use crate::ecs::system_parameters::system_parameter::SystemParameter;
use common::ecs::system::SystemState;
use common::runtime::ApplicationAccess;
use common::world::World;
use std::marker::PhantomData;
use std::sync::mpsc::Sender;

pub struct EventReader<'w, 's, T: Event> {
    events: &'w Events,
    event_cursor: &'s mut usize,
    _marker: PhantomData<T>,
}

impl<'w, 's, T: Event + 'static> EventReader<'w, 's, T> {
    pub fn new(events: &'w Events, event_cursor: &'s mut usize) -> Self {
        Self {
            events,
            event_cursor,
            _marker: PhantomData,
        }
    }

    pub fn read(&mut self) -> impl Iterator<Item = &T> {
        // only read new events
        let events = &self.events.queue[*self.event_cursor..];

        *self.event_cursor = self.events.queue.len();

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
        _access: &'e mut dyn ApplicationAccess,
    ) -> Self::Item<'w, 's, 'e> {
        EventReader::new(
            world.events::<T>(),
            state.event_reader_cursors.entry(T::ID).or_insert(0),
        )
    }
}

pub struct EventWriter<'w, T: Event> {
    queue: &'w mut Events,
    _marker: PhantomData<T>,
}

impl<'w, T: Event + 'static> EventWriter<'w, T> {
    pub fn new(queue: &'w mut Events) -> Self {
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
        _state: &'s mut SystemState,
        _access: &'e mut dyn ApplicationAccess,
    ) -> Self::Item<'w, 's, 'e> {
        EventWriter::new(world.events::<T>())
    }
}

pub struct EventSender<T: Event> {
    sender: Sender<Box<dyn EventMessage>>,
    _marker: PhantomData<T>,
}

impl<T: Event> EventSender<T> {
    fn new(sender: Sender<Box<dyn EventMessage>>) -> Self {
        Self {
            sender,
            _marker: PhantomData,
        }
    }

    pub fn send(&self, event: T) {
        self.sender.send(Box::new(event)).expect("Failed to send an event");
    }
}

impl<T: Event + 'static> SystemParameter for EventSender<T> {
    type Item<'w, 's, 'e> = EventSender<T>;

    fn get<'w, 's, 'e>(
        world: &'w mut World,
        _state: &'s mut SystemState,
        _access: &'e mut dyn ApplicationAccess,
    ) -> Self::Item<'w, 's, 'e> {
        EventSender::new(world.events::<T>().sender.clone())
    }
}
