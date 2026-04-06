mod tests {
    use common::ecs::system_parameters::event::{EventReader, EventWriter};
    use common::engine::scheduler::Scheduler;
    use common::world::World;
    use common_macros::Event;

    #[derive(Event)]
    struct A(u32);

    #[derive(Event)]
    struct B(String);

    #[test]
    fn can_drain_empty_events() {
        fn drain_nothing(mut events: EventReader<A>) {
            assert!(events.drain().next().is_none());
        }

        let mut world = World::default();
        let mut scheduler = Scheduler::default();
        scheduler.register(drain_nothing);
        scheduler.run_systems(&mut world);
    }

    #[test]
    fn can_send_and_read_events() {
        fn send(mut events: EventWriter<A>) {
            events.send(A(1));
            events.send(A(2));
            events.send(A(3));
        }

        fn read(mut events: EventReader<A>) {
            let mut events = events.drain();
            assert_eq!(events.next().unwrap().0, 1);
            assert_eq!(events.next().unwrap().0, 2);
            assert_eq!(events.next().unwrap().0, 3);
            assert!(events.next().is_none());
        }

        let mut world = World::default();
        let mut scheduler = Scheduler::default();
        scheduler.register(send);
        scheduler.register(read);
        scheduler.run_systems(&mut world);
    }
}
