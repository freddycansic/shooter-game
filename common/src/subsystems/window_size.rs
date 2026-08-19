use crate::ecs::subsystem::Subsystem;
use crate::ecs::system_parameters::event::EventReader;
use crate::ecs::system_parameters::res::ResMut;
use crate::engine::scheduler::Stage;
use crate::runtime::RuntimeContext;
use common::engine::scheduler::Scheduler;
use common::world::World;
use common_macros::{Event, Resource};

#[derive(Resource, Copy, Clone)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Event)]
pub struct WindowResized(pub WindowSize);

pub struct WindowSizeSubsystem;

impl WindowSizeSubsystem {
    fn update_window_size(mut resize_event: EventReader<WindowResized>, mut window_size: ResMut<WindowSize>) {
        *window_size = resize_event.read().next().unwrap().0;
    }
}

impl Subsystem for WindowSizeSubsystem {
    fn register_resources(&self, world: &mut World, context: Option<&RuntimeContext>) {
        let window_size = context.unwrap().window.inner_size();

        world.register_resource(WindowSize {
            width: window_size.width,
            height: window_size.height,
        });
    }

    fn register_systems(&self, scheduler: &mut Scheduler) {
        scheduler.register_triggered::<WindowResized, _, _>(Self::update_window_size, Stage::Pre);
    }
}
