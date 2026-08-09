use common::engine::scheduler::Scheduler;
use common::world::World;
use common_macros::{Event, Resource};
use crate::ecs::subsystem::Subsystem;
use crate::ecs::system_parameters::event::EventReader;
use crate::ecs::system_parameters::res::ResMut;
use crate::engine::scheduler::Stage;
use crate::executor::RuntimeContext;

#[derive(Resource, Copy, Clone)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Event)]
pub struct WindowResized(pub WindowSize);

impl Subsystem for WindowSize {
    fn register_resources(world: &mut World, context: Option<&RuntimeContext>) {
        let window_size = context.unwrap().window.inner_size();
        
        world.register_resource(WindowSize { width: window_size.width, height: window_size.height });
    }

    fn register_systems(scheduler: &mut Scheduler) {
        scheduler.register_triggered::<WindowResized, _, _>(update_window_size, Stage::Pre);
    }
}

fn update_window_size(mut resize_event: EventReader<WindowResized>, mut window_size: ResMut<WindowSize>) {
    *window_size = resize_event.read().next().unwrap().0;
}