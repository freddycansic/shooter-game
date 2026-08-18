mod tests {
    use crate::util::{DummyContext, RunNow};
    use common::ecs::system_parameters::event::{EventReader, EventSender, EventWriter};
    use common::ecs::system_parameters::res::{Res, ResMut};
    use common::engine::scheduler::{Scheduler, Stage};
    use common::world::World;
    use common_macros::{Event, Resource};

    #[derive(Event, PartialEq, Debug)]
    struct A(u32);

    #[derive(Event)]
    struct B(String);

    #[test]
    fn can_read_empty_events() {
        fn read_nothing(mut events: EventReader<A>) {
            assert!(events.read().next().is_none());
        }

        let mut world = World::default();
        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(read_nothing, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn can_send_and_read_events() {
        fn send(mut events: EventWriter<A>) {
            events.write(A(1));
            events.write(A(2));
            events.write(A(3));
        }

        fn read(mut events: EventReader<A>) {
            let mut events = events.read();
            assert_eq!(events.next().unwrap().0, 1);
            assert_eq!(events.next().unwrap().0, 2);
            assert_eq!(events.next().unwrap().0, 3);
            assert!(events.next().is_none());
        }

        let mut world = World::default();
        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(send, Stage::Main);
        scheduler.register_continuous(read, Stage::Main);

        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[derive(Resource)]
    struct EventsSent(u32);

    #[test]
    fn can_read_new_events() {
        fn send(mut events: EventWriter<A>, mut events_sent: ResMut<EventsSent>) {
            events_sent.0 += 1;
            events.write(A(events_sent.0));
        }

        fn read(mut events: EventReader<A>, event_sent: Res<EventsSent>) {
            let mut events = events.read();
            let event = events.next();
            assert!(event.is_some());
            assert_eq!(event.unwrap().0, event_sent.0);
            assert!(events.next().is_none());
        }

        let mut world = World::default();
        world.register_resource(EventsSent(0));
        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(send, Stage::Main);
        scheduler.register_continuous(read, Stage::Main);

        for _ in 0..3 {
            scheduler.run(&mut world, &mut DummyContext::default());
        }
    }

    #[test]
    fn can_send_events_from_other_thread() {
        fn send(sender: EventSender<A>) {
            let handle = std::thread::spawn(move || {
                sender.send(A(1));
            });

            handle.join().unwrap();
        }

        fn read(mut reader: EventReader<A>) {
            assert_eq!(reader.read().next(), Some(&A(1)));
        }

        let mut world = World::default();
        let mut scheduler = Scheduler::default();

        scheduler.run_now(send, &mut world);

        scheduler.register_continuous(read, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }
}
