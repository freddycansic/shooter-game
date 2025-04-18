use glium::glutin::surface::WindowSurface;
use glium::Display;
use winit::dpi::LogicalPosition;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorGrabMode, Window};

pub trait Application {
    fn new(window: &Window, display: &Display<WindowSurface>, event_loop: &ActiveEventLoop)
        -> Self;

    fn window_event(
        &mut self,
        event: WindowEvent,
        event_loop: &ActiveEventLoop,
        window: &Window,
        display: &Display<WindowSurface>,
    );

    #[allow(clippy::unused_variables)]
    fn device_event(
        &mut self,
        event: DeviceEvent,
        event_loop: &ActiveEventLoop,
        window: &Window,
        display: &Display<WindowSurface>,
    ) {
    }

    fn capture_cursor(&mut self, window: &Window) {
        window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
            .unwrap();
    }

    fn release_cursor(&mut self, window: &Window) {
        window.set_cursor_grab(CursorGrabMode::None).unwrap();
    }

    fn center_cursor(&mut self, window: &Window) {
        let dimensions = window.inner_size();
        let center = LogicalPosition::new(dimensions.width / 2, dimensions.height / 2);

        window.set_cursor_position(center).unwrap();
    }
}
