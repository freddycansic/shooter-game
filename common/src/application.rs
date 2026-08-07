use glium::Display;
use glium::glutin::surface::WindowSurface;
use winit::dpi::LogicalPosition;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorGrabMode, Window};
use crate::executor::RuntimeExecutor;
use crate::world::World;

pub trait Application {
    fn new(window: &Window, display: &Display<WindowSurface>, event_loop: &ActiveEventLoop) -> Self;

    fn world(&mut self) -> &mut World;

    fn run_systems(&mut self, executor: RuntimeExecutor, display: &Display<WindowSurface>);
    
    fn render(&mut self, event_loop: &ActiveEventLoop, window: &Window, display: &Display<WindowSurface>);

    fn window_event(
        &mut self,
        event: WindowEvent,
        event_loop: &ActiveEventLoop,
        window: &Window,
        display: &Display<WindowSurface>,
    ) {}

    fn device_event(
        &mut self,
        _event: DeviceEvent,
        _event_loop: &ActiveEventLoop,
        _window: &Window,
        _display: &Display<WindowSurface>,
    ) {
    }

    fn new_events(&mut self) {}
}
