mod tests {
    use crate::util::DummyExecutor;
    use common::ecs::system_parameters::commands::Commands;
    use common::engine::scheduler::{Scheduler, Stage};
    use common::world::World;

    #[test]
    fn can_execute_commands() {
        fn system(mut commands: Commands) {
            commands.exit();
            commands.capture_cursor();
            commands.release_cursor();
            commands.center_cursor();
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(system, Stage::Main);

        let mut executor = DummyExecutor::default();
        scheduler.run(&mut World::default(), &mut executor);

        assert_eq!(executor.commands_executed, 4);
    }
}
