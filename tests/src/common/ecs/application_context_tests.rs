mod tests {
    use crate::util::{DummyContext};
    use common::ecs::system_parameters::application_context::ApplicationContext;
    use common::engine::scheduler::{Scheduler, Stage};
    use common::world::World;

    #[test]
    fn can_execute_commands() {
        fn system(mut context: ApplicationContext) {
            context.exit();
            context.capture_cursor();
            context.release_cursor();
            context.center_cursor();
        }

        let mut scheduler = Scheduler::default();
        scheduler.register_continuous(system, Stage::Main);

        let mut context = DummyContext::default();
        scheduler.run(&mut World::default(), &mut context);

        assert_eq!(context.commands_executed, 4);
    }
}
