#[cfg(test)]
mod tests {
    use common::ecs::archetype::Column;
    use common::ecs::component::Component;
    use common::world::World;
    use common_macros::Component;

    #[derive(Component)]
    struct TestComponent(u32);

    #[derive(Component)]
    struct TestComponent2(String);

    #[test]
    fn can_derive_component() {
        assert_ne!(TestComponent::ID.0.len(), 0);
        assert!(TestComponent::ID.0.iter().any(|block| *block > 0));

        assert_ne!(TestComponent2::ID.0.len(), 0);
        assert!(TestComponent2::ID.0.iter().any(|block| *block > 0));

        assert_ne!(TestComponent::ID, TestComponent2::ID);
    }

    #[test]
    fn can_make_empty_column() {
        let column = Column::new_empty(TestComponent::ID);

        assert!(column.components.get().is_none());
        assert_eq!(column.id, TestComponent::ID);
    }

    #[test]
    fn can_find_archetype() {
        let mut world = World::default();

        let archetype = world.find_archetype::<TestComponent>();
        assert_eq!(archetype.columns.len(), 1);
        assert_eq!(archetype.columns[0].id, TestComponent::ID);
        assert!(archetype.entities.is_empty());
    }

    #[test]
    fn can_spawn_single() {
        let mut world = World::default();
        let entity = world.spawn(TestComponent(1));

        let archetype = world.find_archetype::<TestComponent>();
        assert_eq!(archetype.columns.len(), 1);
        assert_eq!(archetype.columns[0].id, TestComponent::ID);
        assert_eq!(archetype.entities.len(), 1);
        assert_eq!(archetype.entities[0], entity);
    }

    #[test]
    fn can_spawn_double() {
        let mut world = World::default();
        let entity = world.spawn((TestComponent(1234), TestComponent2("first".to_string())));

        let archetype = world.find_archetype::<(TestComponent, TestComponent2)>();
        assert_eq!(archetype.columns.len(), 2);

        let column_1 = archetype.components_of_type::<TestComponent>().unwrap();
        assert_eq!(column_1.len(), 1);
        assert_eq!(column_1[0].0, 1234);

        let column_2 = archetype.components_of_type::<TestComponent2>().unwrap();
        assert_eq!(column_2.len(), 1);
        assert_eq!(column_2[0].0, "first");
    }

    #[test]
    fn can_read_components_from_archetype() {
        let mut world = World::default();
        world.spawn(TestComponent(1));
        world.spawn(TestComponent(2));
        world.spawn(TestComponent(3));

        let archetype = world.find_archetype::<TestComponent>();
        let column = archetype.components_of_type::<TestComponent>().unwrap();
        assert_eq!(column.len(), 3);
        assert_eq!(column[0].0, 1);
        assert_eq!(column[1].0, 2);
        assert_eq!(column[2].0, 3);
    }

    #[test]
    fn can_write_components_in_archetype() {
        let mut world = World::default();
        world.spawn(TestComponent(1));
        world.spawn(TestComponent(2));
        world.spawn(TestComponent(3));

        let archetype = world.find_archetype::<TestComponent>();
        let column = archetype.components_of_type_mut::<TestComponent>().unwrap();
        assert_eq!(column.len(), 3);
        assert_eq!(column[0].0, 1);
        assert_eq!(column[1].0, 2);
        assert_eq!(column[2].0, 3);

        column[0] = TestComponent(6);
        column[1] = TestComponent(7);
        column[2] = TestComponent(8);

        assert_eq!(column.len(), 3);
        assert_eq!(column[0].0, 6);
        assert_eq!(column[1].0, 7);
        assert_eq!(column[2].0, 8);
    }
}
