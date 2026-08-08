#[cfg(test)]
mod tests {
    use crate::util::DummyExecutor;
    use common::ecs::resource::Resource;
    use common::ecs::system_parameters::res::{Res, ResMut};
    use common::engine::engine::Engine;
    use common::engine::scheduler::Scheduler;
    use common::world::World;
    use common_macros::Resource;

    #[derive(Resource)]
    struct A(u32);

    #[derive(Resource)]
    struct B(u32);

    #[test]
    fn can_register_resources() {
        let mut world = World::default();
        world.register_resource(A(1));
        world.register_resource(B(2));

        assert_eq!(world.resources.len(), 2);
    }

    #[test]
    #[should_panic]
    fn cannot_register_resource_twice() {
        let mut world = World::default();
        world.register_resource(A(1));
        world.register_resource(A(2));
    }

    #[test]
    fn can_read_resource() {
        let mut world = World::default();
        world.register_resource(A(1));

        fn read_resources(res: Res<A>) {
            assert_eq!(res.0, 1);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(read_resources);
        scheduler.run_systems(&mut world, &mut DummyExecutor::default());
    }

    #[test]
    fn can_write_resource() {
        let mut world = World::default();
        world.register_resource(A(1));

        fn read_resource_before(res: Res<A>) {
            assert_eq!(res.0, 1);
        }

        fn write_resource(mut res: ResMut<A>) {
            res.0 = 5;
        }

        fn read_resource_after(res: Res<A>) {
            assert_eq!(res.0, 5);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(read_resource_before);
        scheduler.register_continuous(write_resource);
        scheduler.register_continuous(read_resource_after);
        scheduler.run_systems(&mut world, &mut DummyExecutor::default());
    }
}
