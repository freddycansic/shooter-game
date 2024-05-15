use std::fs;

use color_eyre::Result;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Program};
use winit::dpi::LogicalPosition;
use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowBuilder};

#[derive(Debug)]
pub struct OpenGLContext {
    pub window: Window,
    pub display: Display<WindowSurface>,
}

impl OpenGLContext {
    pub fn new(title: &str, fullscreen: bool, event_loop: &EventLoop<()>) -> Self {
        let mut window_builder = WindowBuilder::new().with_title(title);

        if fullscreen {
            window_builder = window_builder.with_fullscreen(Some(Fullscreen::Borderless(None)));
        } else {
            window_builder = window_builder.with_maximized(true);
        }

        let (window, display) = SimpleWindowBuilder::new()
            .set_window_builder(window_builder)
            .build(event_loop);

        Self { window, display }
    }

    pub fn capture_cursor(&mut self) {
        self.window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Locked))
            .unwrap();
    }

    pub fn release_cursor(&mut self) {
        self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
    }

    pub fn center_cursor(&mut self) {
        let dimensions = self.window.inner_size();
        let center = LogicalPosition::new(dimensions.width / 2, dimensions.height / 2);

        self.window.set_cursor_position(center).unwrap();
    }
}

    pub fn new_program(
        vertex_source_path: &str,
        fragment_source_path: &str,
        geometry_source_path: Option<&str>,
        display: &Display<WindowSurface>,
    ) -> Result<Program> {
        let vertex_source = fs::read_to_string(vertex_source_path)?;
        let fragment_source = fs::read_to_string(fragment_source_path)?;
        let geometry_source = geometry_source_path.map(|path| fs::read_to_string(path).unwrap());

        Ok(Program::from_source(
            display,
            vertex_source.as_str(),
            fragment_source.as_str(),
            geometry_source.as_deref(),
        )?)
    }
