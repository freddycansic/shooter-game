#[cfg(test)]
mod tests {
    use common::ecs::system_parameters::event::{EventReader, EventWriter};
    use common::ecs::system_parameters::res::ResMut;
    use common::engine::scheduler::Scheduler;
    use common::world::World;
    use common_macros::{Event, Resource};

    #[derive(Event)]
    struct TriggerEvent;

    #[derive(Event)]
    struct TriggerEvent2;

    #[test]
    fn non_triggered_events_dont_run() {
        fn dont_run_me() {
            assert!(false);
        }

        let mut world = World::default();
        let mut scheduler = Scheduler::default();

        scheduler.register_triggered::<TriggerEvent, _, _>(dont_run_me);
        scheduler.run_systems(&mut world);
    }

    #[derive(Resource)]
    struct TimesRan1(u32);

    #[derive(Resource)]
    struct TimesRan2(u32);

    #[test]
    fn one_trigger_runs_system_once() {
        let mut world = World::default();
        world.register_resource(TimesRan1(0));

        fn run_me(mut times_ran: ResMut<TimesRan1>, mut events: EventReader<TriggerEvent>) {
            times_ran.0 += 1;

            assert_eq!(events.read().count(), 1);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_triggered::<TriggerEvent, _, _>(run_me);

        world.write_event(TriggerEvent);
        scheduler.run_systems(&mut world);

        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 1);
    }

    #[test]
    fn multiple_triggers_runs_system_once() {
        let mut world = World::default();
        world.register_resource(TimesRan1(0));

        fn run_me(mut times_ran: ResMut<TimesRan1>, mut events: EventReader<TriggerEvent>) {
            times_ran.0 += 1;

            assert_eq!(events.read().count(), 5);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_triggered::<TriggerEvent, _, _>(run_me);

        for _ in 0..5 {
            world.write_event(TriggerEvent);
        }

        scheduler.run_systems(&mut world);

        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 1);
    }

    #[test]
    fn one_trigger_runs_multiple_systems_once() {
        let mut world = World::default();
        world.register_resource(TimesRan1(0));
        world.register_resource(TimesRan2(0));

        fn run_me_1(mut times_ran: ResMut<TimesRan1>, mut events: EventReader<TriggerEvent>) {
            times_ran.0 += 1;

            assert_eq!(events.read().count(), 1);
        }

        fn run_me_2(mut times_ran: ResMut<TimesRan2>, mut events: EventReader<TriggerEvent>) {
            times_ran.0 += 1;

            assert_eq!(events.read().count(), 1);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_triggered::<TriggerEvent, _, _>(run_me_1);
        scheduler.register_triggered::<TriggerEvent, _, _>(run_me_2);

        world.write_event(TriggerEvent);
        scheduler.run_systems(&mut world);

        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 1);
        assert_eq!(world.resource::<TimesRan2>().unwrap().0, 1);
    }

    #[test]
    fn multiple_triggers_runs_multiple_systems_once() {
        let mut world = World::default();
        world.register_resource(TimesRan1(0));
        world.register_resource(TimesRan2(0));

        fn run_me_1(mut times_ran: ResMut<TimesRan1>, mut events: EventReader<TriggerEvent>) {
            times_ran.0 += 1;

            assert_eq!(events.read().count(), 5);
        }

        fn run_me_2(mut times_ran: ResMut<TimesRan2>, mut events: EventReader<TriggerEvent>) {
            times_ran.0 += 1;

            assert_eq!(events.read().count(), 5);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_triggered::<TriggerEvent, _, _>(run_me_1);
        scheduler.register_triggered::<TriggerEvent, _, _>(run_me_2);

        for _ in 0..5 {
            world.write_event(TriggerEvent);
        }

        scheduler.run_systems(&mut world);

        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 1);
        assert_eq!(world.resource::<TimesRan2>().unwrap().0, 1);
    }

    #[test]
    fn system_can_trigger_same_event() {
        let mut world = World::default();
        world.register_resource(TimesRan1(0));

        fn trigger_self(mut times_ran: ResMut<TimesRan1>, mut events: EventWriter<TriggerEvent>) {
            times_ran.0 += 1;
            events.write(TriggerEvent);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_triggered::<TriggerEvent, _, _>(trigger_self);

        world.write_event(TriggerEvent);
        scheduler.run_systems(&mut world);
        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 1);

        scheduler.run_systems(&mut world);
        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 2);
    }

    #[test]
    fn one_system_triggered_by_different_events() {
        let mut world = World::default();
        world.register_resource(TimesRan1(0));

        fn system(mut times_ran: ResMut<TimesRan1>) {
            times_ran.0 += 1;
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_triggered::<TriggerEvent, _, _>(system);
        scheduler.register_triggered::<TriggerEvent2, _, _>(system);

        world.write_event(TriggerEvent);
        scheduler.run_systems(&mut world);
        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 1);

        world.write_event(TriggerEvent2);
        scheduler.run_systems(&mut world);
        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 2);

        world.write_event(TriggerEvent);
        world.write_event(TriggerEvent2);
        scheduler.run_systems(&mut world);
        assert_eq!(world.resource::<TimesRan1>().unwrap().0, 3);
    }
}
