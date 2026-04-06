mod tests {
    use common::world::World;
    use common_macros::Event;

    #[derive(Event)]
    struct A(u32);

    #[derive(Event)]
    struct B(u32);

    #[test]
    fn can_dispatch_to_no_callbacks() {
        let mut world = World::default();

        world.dispatch(A(3));
    }

    #[test]
    fn can_dispatch_to_one_callback() {
        let mut world = World::default();
        world.add_callback(|event: &A| assert_eq!(event.0, 1));
        world.dispatch(A(1));
    }

    #[test]
    fn can_dispatch_to_multiple_callbacks() {
        let mut world = World::default();

        world.add_callback(|event: &A| assert_eq!(event.0, 1));
        world.add_callback(|event: &A| assert_eq!(event.0, 1));
        world.add_callback(|event: &A| assert_eq!(event.0, 1));
        world.dispatch(A(1));
    }
}
