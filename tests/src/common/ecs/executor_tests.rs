mod tests {
    use common::ecs::system_parameters::commands::Commands;
    use common::engine::scheduler::Scheduler;
    use common::world::World;
    use crate::util::DummyExecutor;

    #[test]
    fn can_execute_commands() {
        fn system(mut commands: Commands) {
            commands.exit();
            commands.capture_cursor();
            commands.release_cursor();
            commands.center_cursor();
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(system);

        let mut executor = DummyExecutor::default();
        scheduler.run_systems(&mut World::default(), &mut executor);

        assert_eq!(executor.commands_executed, 4);
    }
}