#[cfg(test)]
mod tests {
    use common::ecs::system::Query;
    use common::world::World;
    use common_macros::Component;
    use common::engine::engine::Engine;

    #[derive(Component)]
    struct Position;

    fn system(q1: Query<Position>) {
        println!("Hello");
    }

    #[test]
    fn test_register_system() {
        let mut world = World::default();
        let mut engine = Engine::new();
        engine.register(system);

        engine.run_systems(&mut world);
    }
}
