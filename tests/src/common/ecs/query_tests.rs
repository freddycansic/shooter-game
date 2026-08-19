#[cfg(test)]
mod tests {
    use crate::util::DummyContext;
    use common::ecs::system_parameters::query::Query;
    use common::engine::scheduler::{Scheduler, Stage};
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
        scheduler.register_continuous(system_one_parameter, Stage::Main);

        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn test_register_system_two_parameters() {
        let mut world = World::default();
        let mut scheduler = Scheduler::default();

        fn system_two_parameters(q1: Query<&Position>, q2: Query<&Velocity>) {}
        scheduler.register_continuous(system_two_parameters, Stage::Main);

        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[derive(Component, PartialEq, Debug)]
    struct A(u32);

    #[derive(Component, PartialEq, Debug)]
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
        scheduler.register_continuous(system_query_iterator, Stage::Main);

        scheduler.run(&mut world, &mut DummyContext::default());
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
        scheduler.register_continuous(system_query_iterator_overlapping, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn test_query_empty_world() {
        let mut world = World::default();

        fn system(mut q: Query<&A>) {
            let result = q.iter().collect::<Vec<&A>>();
            assert_eq!(result.len(), 0);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(system, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
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
        scheduler.register_continuous(system, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
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
        scheduler.register_continuous(system, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
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
        scheduler.register_continuous(system, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
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
        scheduler.register_continuous(system_fetch_mismatch, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn test_query_mutable() {
        let mut world = World::default();

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        fn set_to_five(mut q: Query<&mut A>) {
            for a in q.iter() {
                a.0 = 5;
            }
        }

        fn check_are_five(mut q: Query<&A>) {
            for a in q.iter() {
                assert_eq!(a.0, 5);
            }
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(set_to_five, Stage::Main);
        scheduler.register_continuous(check_are_five, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn test_query_mixed_access() {
        let mut world = World::default();

        world.spawn((A(1), B(4)));
        world.spawn((A(2), B(5)));
        world.spawn((A(3), B(6)));

        fn read_and_modify(mut q: Query<(&mut A, &B)>) {
            let mut iter = q.iter();
            let (a, b) = iter.next().unwrap();
            assert_eq!(a.0, 1);
            assert_eq!(b.0, 4);
            a.0 = 7;
            let (a, b) = iter.next().unwrap();
            assert_eq!(a.0, 2);
            assert_eq!(b.0, 5);
            a.0 = 8;
            let (a, b) = iter.next().unwrap();
            assert_eq!(a.0, 3);
            assert_eq!(b.0, 6);
            a.0 = 9;
        }

        fn check_are_modified(mut q: Query<&A>) {
            let mut iter = q.iter();
            assert_eq!(iter.next().unwrap().0, 7);
            assert_eq!(iter.next().unwrap().0, 8);
            assert_eq!(iter.next().unwrap().0, 9);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(read_and_modify, Stage::Main);
        scheduler.register_continuous(check_are_modified, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn optional_component_exists() {
        let mut world = World::default();

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        fn optional_a(mut query: Query<Option<&A>>) {
            let mut iter = query.iter();
            assert_eq!(iter.next().unwrap(), Some(&A(1)));
            assert_eq!(iter.next().unwrap(), Some(&A(2)));
            assert_eq!(iter.next().unwrap(), Some(&A(3)));
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(optional_a, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn optional_component_doesnt_exist() {
        let mut world = World::default();

        world.spawn(B(1));
        world.spawn(B(2));
        world.spawn(B(3));

        fn optional_a(mut query: Query<Option<&A>>) {
            let mut iter = query.iter();
            assert_eq!(iter.next().unwrap(), None);
            assert_eq!(iter.next().unwrap(), None);
            assert_eq!(iter.next().unwrap(), None);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(optional_a, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn required_component_exists_and_optional_doesnt() {
        let mut world = World::default();

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        fn optional_a(mut query: Query<(&A, Option<&B>)>) {
            let mut iter = query.iter();
            assert_eq!(iter.next().unwrap(), (&A(1), None));
            assert_eq!(iter.next().unwrap(), (&A(2), None));
            assert_eq!(iter.next().unwrap(), (&A(3), None));
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(optional_a, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn optional_component_exists_and_required_doesnt() {
        let mut world = World::default();

        world.spawn(B(1));
        world.spawn(B(2));
        world.spawn(B(3));

        fn optional_a(mut query: Query<(&A, Option<&B>)>) {
            let mut iter = query.iter();
            assert_eq!(iter.next(), None);
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(optional_a, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }

    #[test]
    fn some_optional_components_exists() {
        let mut world = World::default();

        world.spawn(A(1));
        world.spawn((A(2), B(4)));
        world.spawn(A(3));

        fn optional_a(mut query: Query<(&A, Option<&B>)>) {
            // The order shouldn't matter
            let mut iter = query.iter();
            assert_eq!(iter.next().unwrap(), (&A(1), None));
            assert_eq!(iter.next().unwrap(), (&A(3), None));
            assert_eq!(iter.next().unwrap(), (&A(2), Some(&B(4))));
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(optional_a, Stage::Main);
        scheduler.run(&mut world, &mut DummyContext::default());
    }
}
