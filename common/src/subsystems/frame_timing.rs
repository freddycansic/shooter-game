use crate::engine::scheduler::{Scheduler, Stage};
use crate::runtime::RuntimeContext;
use common::ecs::subsystem::Subsystem;
use common::ecs::system_parameters::res::ResMut;
use common::world::World;
use common_macros::{Event, Resource};
use std::time::Instant;

#[derive(Event)]
pub struct WinitNewEvents;

#[derive(Resource)]
pub struct FrameTiming {
    pub last_frame_end: Instant,
    pub frame_count: u128,
    pub deltatime: f64,
    pub fps: f32,
}

pub fn update_statistics(mut frame_state: ResMut<FrameTiming>) {
    frame_state.frame_count = (frame_state.frame_count + 1) % u128::MAX;

    frame_state.deltatime = frame_state.last_frame_end.elapsed().as_secs_f64();
    frame_state.fps = (1.0 / frame_state.deltatime) as f32;

    frame_state.last_frame_end = Instant::now();
}

pub struct FrameTimingSubsystem;

impl Subsystem for FrameTimingSubsystem {
    fn register_resources(&self, world: &mut World, _context: Option<&RuntimeContext>) {
        let state = FrameTiming {
            last_frame_end: Instant::now(),
            frame_count: 0,
            deltatime: 0.0,
            fps: 0.0,
        };

        world.register_resource(state);
    }

    fn register_systems(&self, scheduler: &mut Scheduler) {
        scheduler.register_triggered::<WinitNewEvents, _, _>(update_statistics, Stage::Pre);
    }
}
