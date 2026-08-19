mod tests {
    use crate::util::DummyContext;
    use common::ecs::system_parameters::res::ResMut;
    use common::engine::scheduler::{Scheduler, Stage, SystemOrder};
    use common::world::World;
    use common_macros::Resource;

    #[derive(Resource)]
    struct WhoRanLast(u32);

    #[test]
    fn can_run_systems_in_order() {
        fn first(mut who_ran_last: ResMut<WhoRanLast>) {
            assert_eq!(who_ran_last.0, 0);
            who_ran_last.0 = 1;
        }

        fn second(mut who_ran_last: ResMut<WhoRanLast>) {
            assert_eq!(who_ran_last.0, 1);
            who_ran_last.0 = 2;
        }

        fn third(mut who_ran_last: ResMut<WhoRanLast>) {
            assert_eq!(who_ran_last.0, 2);
            who_ran_last.0 = 3;
        }

        let mut world = World::default();
        world.register_resource(WhoRanLast(0));

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous_order(SystemOrder::first(first).then(second).then(third), Stage::Main);

        scheduler.run(&mut world, &mut DummyContext::default());
        assert_eq!(world.resource::<WhoRanLast>().unwrap().0, 3);
    }
}
