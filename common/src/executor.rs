use winit::dpi::LogicalPosition;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorGrabMode, Window};

pub struct RuntimeExecutor<'a> {
    pub window: &'a mut Window,
    event_loop: &'a ActiveEventLoop,
}

impl<'a> RuntimeExecutor<'a> {
    pub fn new(window: &'a mut Window, event_loop: &'a ActiveEventLoop) -> Self {
        Self { window, event_loop }
    }
}

pub trait CommandExecutor {
    fn capture_cursor(&mut self);
    fn release_cursor(&mut self);
    fn center_cursor(&mut self);
    fn exit(&mut self);
}

impl<'a> CommandExecutor for RuntimeExecutor<'a> {
    fn capture_cursor(&mut self) {
        self.window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined))
            .unwrap();

        self.window.set_cursor_visible(false);
    }

    fn release_cursor(&mut self) {
        self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        self.window.set_cursor_visible(true);
    }

    fn center_cursor(&mut self) {
        let dimensions = self.window.inner_size();
        let center = LogicalPosition::new(dimensions.width / 2, dimensions.height / 2);

        match self.window.set_cursor_position(center) {
            Ok(_) => (),
            Err(e) => log::warn!("Failed to set cursor position: {:?}", e),
        }
    }

    fn exit(&mut self) {
        self.event_loop.exit();
    }
}
