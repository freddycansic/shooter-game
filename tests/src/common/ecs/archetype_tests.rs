#[cfg(test)]
mod tests {
    use common::ecs::component::Component;
    use common_macros::Component;
    
    #[derive(Component)]
    struct TestComponent(u32);

    #[derive(Component)]
    struct TestComponent2(String);

    #[test]
    fn can_derive_component() {
        assert_ne!(TestComponent::ID.0, 0);
        assert_ne!(TestComponent2::ID.0, 0);
        assert_ne!(TestComponent::ID, TestComponent2::ID);
    }
}
