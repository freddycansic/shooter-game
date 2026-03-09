#[cfg(test)]
mod tests {
    use common::ecs::system::Query;
    use common::world::World;
    use common_macros::Component;

    #[derive(Component)]
    struct Position;

    fn system(q1: Query<Position>) {}

    #[test]
    fn test_register_system() {
        let mut world = World::default();
        world.systems.register(system);
    }
}
