use crate::engine::assets::Assets;
use crate::engine::input::Input;
use crate::engine::scheduler::Scheduler;
use crate::executor::RuntimeContext;
use crate::subsystems::frame_timing::FrameTiming;
use crate::subsystems::window_size::WindowSize;
use crate::world::World;
use common::ecs::subsystem::Subsystem;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use crate::engine::renderer::Renderer;

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

    fn register_subsystem_with_context<S>(&mut self, context: &RuntimeContext)
    where
        S: Subsystem,
    {
        S::register_resources(&mut self.world(), Some(context));
        S::register_systems(&mut self.scheduler());
    }

    fn register_subsystem<S>(&mut self)
    where
        S: Subsystem,
    {
        S::register_resources(&mut self.world(), None);
        S::register_systems(&mut self.scheduler());
    }

    fn register_core_ecs_state(&mut self, context: &RuntimeContext) {
        self.register_subsystem_with_context::<Assets>(&context);
        self.register_subsystem_with_context::<WindowSize>(&context);
        self.register_subsystem_with_context::<Renderer>(&context);

        self.register_subsystem::<Input>();
        self.register_subsystem::<FrameTiming>();
    }

    fn world(&mut self) -> &mut World;

    fn scheduler(&mut self) -> &mut Scheduler;
}
