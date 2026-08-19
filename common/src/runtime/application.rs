use crate::ecs::subsystem::Subsystem;
use crate::engine::assets::{Assets, AssetsSubsystem};
use crate::engine::input::{Input, InputSubsystem};
use crate::engine::renderer::Renderer;
use crate::engine::scheduler::Scheduler;
use crate::runtime::RuntimeContext;
use crate::subsystems::frame_timing::{FrameTiming, FrameTimingSubsystem};
use crate::subsystems::window_size::{WindowSize, WindowSizeSubsystem};
use crate::world::World;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use common::engine::renderer::RendererSubsystem;

pub trait Application {
    fn new(context: &RuntimeContext) -> Self;

    fn run(&mut self, context: RuntimeContext);

    fn window_event(
        &mut self,
        _event: WindowEvent,
        _event_loop: &ActiveEventLoop,
        _window: &Window,
        _display: &Display<WindowSurface>,
    ) {
    }

    fn device_event(
        &mut self,
        _event: DeviceEvent,
        _event_loop: &ActiveEventLoop,
        _window: &Window,
        _display: &Display<WindowSurface>,
    ) {
    }

    fn new_events(&mut self) {}

    fn register_subsystem_with_context<S>(&mut self, subsystem: S, context: &RuntimeContext)
    where
        S: Subsystem,
    {
        subsystem.register_resources(&mut self.world(), Some(context));
        subsystem.register_systems(&mut self.scheduler());
    }

    fn register_subsystem<S>(&mut self, subsystem: S)
    where
        S: Subsystem,
    {
        subsystem.register_resources(&mut self.world(), None);
        subsystem.register_systems(&mut self.scheduler());
    }

    fn register_core_ecs_state(&mut self, context: &RuntimeContext) {
        self.register_subsystem_with_context(AssetsSubsystem, &context);
        self.register_subsystem_with_context(WindowSizeSubsystem, &context);
        self.register_subsystem_with_context(RendererSubsystem, &context);

        self.register_subsystem(InputSubsystem);
        self.register_subsystem(FrameTimingSubsystem);
    }

    fn world(&mut self) -> &mut World;

    fn scheduler(&mut self) -> &mut Scheduler;
}
