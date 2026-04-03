#[cfg(test)]
mod tests {
    use common::ecs::system_parameters::query::Query;
    use common::engine::scheduler::Scheduler;
    use common::world::World;
    use common_macros::Component;

    #[derive(Component)]
    struct Position;

    #[derive(Component)]
    struct Velocity;

    #[test]
    fn test_register_system_one_parameter() {
        let mut world = World::default();
        let mut scheduler = Scheduler::default();

        fn system_one_parameter(q1: Query<&Position>) {}
        scheduler.register(system_one_parameter);

        scheduler.run_systems(&mut world);
    }

    #[test]
    fn test_register_system_two_parameters() {
        let mut world = World::default();
        let mut scheduler = Scheduler::default();

        fn system_two_parameters(q1: Query<&Position>, q2: Query<&Velocity>) {}
        scheduler.register(system_two_parameters);

        scheduler.run_systems(&mut world);
    }

    #[derive(Component)]
    struct A(u32);

    #[derive(Component)]
    struct B(u32);

    #[test]
    fn test_query_iterator_single() {
        let mut world = World::default();

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        let mut scheduler = Scheduler::default();

        fn system_query_iterator(mut q: Query<&A>) {
            let mut comp_data = q.iter().map(|comp| comp.0).collect::<Vec<u32>>();
            comp_data.sort();
            assert_eq!(comp_data, vec![1, 2, 3]);
        }
        scheduler.register(system_query_iterator);

        scheduler.run_systems(&mut world);
    }

    #[test]
    fn test_query_iterator_overlapping() {
        let mut world = World::default();

        world.spawn((A(1), B(6)));
        world.spawn((A(2), B(7)));
        world.spawn((A(3), B(8)));
        world.spawn(A(4));
        world.spawn(A(5));

        fn system_query_iterator_overlapping(mut q: Query<&A>) {
            let mut comp_data = q.iter().map(|comp| comp.0).collect::<Vec<u32>>();
            comp_data.sort();
            assert_eq!(comp_data, vec![1, 2, 3, 4, 5]);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register(system_query_iterator_overlapping);
        scheduler.run_systems(&mut world);
    }

    #[test]
    fn test_query_empty_world() {
        let mut world = World::default();

        fn system(mut q: Query<&A>) {
            let result = q.iter().collect::<Vec<&A>>();
            assert_eq!(result.len(), 0);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register(system);
        scheduler.run_systems(&mut world);
    }

    #[derive(Component)]
    struct Health(u32);

    #[test]
    fn test_query_mixed_components() {
        let mut world = World::default();

        world.spawn(A(1));
        world.spawn((A(2), B(10)));
        world.spawn((A(3), Health(99)));

        fn system(mut q: Query<&A>) {
            let mut values: Vec<u32> = q.iter().map(|c| c.0).collect();
            values.sort();
            assert_eq!(values, vec![1, 2, 3]);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register(system);
        scheduler.run_systems(&mut world);
    }

    #[test]
    fn test_query_two_components_exclusive() {
        let mut world = World::default();

        world.spawn((A(1), B(10)));
        world.spawn((A(2), B(20)));
        world.spawn(A(3)); // should be excluded

        fn system(mut q: Query<(&A, &B)>) {
            let result: Vec<(u32, u32)> = q.iter().map(|(a, b)| (a.0, b.0)).collect();

            assert_eq!(result.len(), 2);
            assert!(result.contains(&(1, 10)));
            assert!(result.contains(&(2, 20)));
        }

        let mut scheduler = Scheduler::default();
        scheduler.register(system);
        scheduler.run_systems(&mut world);
    }

    #[test]
    fn test_query_spawned_both_ways() {
        let mut world = World::default();

        world.spawn((A(1), B(5)));
        world.spawn((B(6), A(2)));

        fn system(mut q: Query<&A>) {
            let mut values: Vec<u32> = q.iter().map(|c| c.0).collect();
            values.sort();
            assert_eq!(values, vec![1, 2]);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register(system);
        scheduler.run_systems(&mut world);
    }

    #[test]
    fn test_query_different_order() {
        let mut world = World::default();

        world.spawn((A(1), B(5)));
        world.spawn((B(6), A(2)));
        world.spawn((A(3), B(7)));
        world.spawn((B(8), A(4)));

        fn system_fetch_mismatch(mut q: Query<(&A, &B)>) {
            let mut values = q.iter().map(|(a, b)| (a.0, b.0)).collect::<Vec<(u32, u32)>>();
            values.sort_by(|first, second| first.0.cmp(&second.0)); // sort on "a"

            assert_eq!(values, vec![(1, 5), (2, 6), (3, 7), (4, 8)]);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register(system_fetch_mismatch);
        scheduler.run_systems(&mut world);
    }
}
