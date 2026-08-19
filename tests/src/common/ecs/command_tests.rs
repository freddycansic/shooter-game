mod tests {
    use crate::util::{DummyContext, RunNow};
    use common::ecs::entity::Entity;
    use common::ecs::system_parameters::commands::Commands;
    use common::ecs::system_parameters::query::Query;
    use common::engine::scheduler::{Scheduler, Stage};
    use common::world::World;
    use common_macros::Component;

    #[derive(Component, Debug, PartialEq)]
    struct A(i32);

    #[test]
    fn can_spawn_entity() {
        fn spawn(mut commands: Commands) {
            commands.spawn(A(6));
            commands.spawn(A(7));
            commands.spawn(A(1));
        }

        fn assert_not_spawned(mut query: Query<&A>) {
            assert_eq!(query.iter().next(), None);
        }

        fn assert_spawned(mut query: Query<&A>) {
            let mut iter = query.iter();
            assert_eq!(iter.next(), Some(&A(6)));
            assert_eq!(iter.next(), Some(&A(7)));
            assert_eq!(iter.next(), Some(&A(1)));
        }

        let mut world = World::default();
        let scheduler = Scheduler::default();

        scheduler.run_now(spawn, &mut world);
        scheduler.run_now(assert_not_spawned, &mut world);

        world.execute_command_queue();

        scheduler.run_now(assert_spawned, &mut world);
    }

    #[test]
    fn can_destroy_entity() {
        fn destroy(mut commands: Commands, mut query: Query<(Entity, &A)>) {
            for (entity, _) in query.iter() {
                commands.destroy(entity);
            }
        }

        fn assert_destroyed(mut query: Query<&A>) {
            let mut iter = query.iter();
            assert_eq!(iter.next(), None);
        }

        let mut world = World::default();
        let scheduler = Scheduler::default();

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        scheduler.run_now(destroy, &mut world);
        world.execute_command_queue();
        scheduler.run_now(assert_destroyed, &mut world);
    }
}
