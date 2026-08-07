use crate::ecs::system_parameters::event::EventReader;
use crate::engine::scheduler::Scheduler;
use common::ecs::subsystem::Subsystem;
use common::ecs::system_parameters::res::ResMut;
use common::engine::engine::Engine;
use common::world::World;
use common_macros::{Event, Resource};
use std::time::Instant;

pub struct FrameTiming;

#[derive(Event)]
pub struct WinitNewEvents;

#[derive(Resource)]
pub struct FrameState {
    pub last_frame_end: Instant,
    pub frame_count: u128,
    pub deltatime: f64,
    pub fps: f32,
    pub gui: GuiState,
}

// TODO move gui state into its own resource
struct GuiState {
    pub render_lights: bool,
    pub debug_cube_index: usize,
    pub debug_cube_opacity: f32,
    pub render_debug_mouse_rays: bool,
}

pub fn update_statistics(mut frame_state: ResMut<FrameState>) {
    frame_state.frame_count = (frame_state.frame_count + 1) % u128::MAX;

    frame_state.deltatime = frame_state.last_frame_end.elapsed().as_secs_f64();
    frame_state.fps = (1.0 / frame_state.deltatime) as f32;

    frame_state.last_frame_end = Instant::now();
}

impl Subsystem for FrameTiming {
    fn register_resources(world: &mut World) {
        let state = FrameState {
            last_frame_end: Instant::now(),
            frame_count: 0,
            deltatime: 0.0,
            fps: 0.0,
            gui: GuiState {
                render_lights: true,
                debug_cube_index: 0,
                debug_cube_opacity: 0.5,
                render_debug_mouse_rays: false,
            },
        };

        world.register_resource(state);
    }

    fn register_systems(scheduler: &mut Scheduler) {
        scheduler.register_triggered::<WinitNewEvents, _, _>(update_statistics);
    }
}
