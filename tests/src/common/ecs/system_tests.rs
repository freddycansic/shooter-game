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

    fn system_one_parameter(q1: Query<Position>) {}

    #[test]
    fn test_register_system_one_parameter() {
        let mut world = World::default();
        let mut engine = Scheduler::default();
        engine.register(system_one_parameter);

        engine.run_systems(&mut world);
    }

    fn system_two_parameters(q1: Query<Position>, q2: Query<Velocity>) {}

    #[test]
    fn test_register_system_two_parameters() {
        let mut world = World::default();
        let mut engine = Scheduler::default();
        engine.register(system_two_parameters);

        engine.run_systems(&mut world);
    }
}
