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

    fn system_one_parameter(q1: Query<&Position>) {}

    #[test]
    fn test_register_system_one_parameter() {
        let mut world = World::default();
        let mut engine = Scheduler::default();
        engine.register(system_one_parameter);

        engine.run_systems(&mut world);
    }

    fn system_two_parameters(q1: Query<&Position>, q2: Query<&Velocity>) {}

    #[test]
    fn test_register_system_two_parameters() {
        let mut world = World::default();
        let mut engine = Scheduler::default();
        engine.register(system_two_parameters);

        engine.run_systems(&mut world);
    }

    #[derive(Component)]
    struct ImportantComponent(u32);

    fn system_query_iterator(q: Query<&ImportantComponent>) {
        for comp in q.iter() {
            
        }
    }

    #[test]
    fn test_query_iterator() {
        let mut world = World::default();

        world.spawn(ImportantComponent(1));
        world.spawn(ImportantComponent(2));
        world.spawn(ImportantComponent(3));

        let mut engine = Scheduler::default();
        engine.register(system_two_parameters);

        engine.run_systems(&mut world);
    }
}
