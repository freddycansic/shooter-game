use std::fs;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::{Display, Program};
use glium::glutin::surface::WindowSurface;
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::window::Window;

use color_eyre::Result;

pub struct OpenGLContext {
    pub window: Window,
    pub display: Display<WindowSurface>,
}

impl OpenGLContext {
    pub fn new(title: &str, event_loop: &EventLoop<()>) -> Result<Self> {
        let (window, display) = SimpleWindowBuilder::new().with_title(title).build(event_loop);

        Ok(Self {
            window,
            display,
        })
    }
}

pub struct RenderingContext {
    pub program: Program
}

impl RenderingContext {
    pub fn new(vertex_source_path: &str, fragment_source_path: &str, display: &Display<WindowSurface>) -> Result<Self> {
        let vertex_source = fs::read_to_string(vertex_source_path)?;
        let fragment_source = fs::read_to_string(fragment_source_path)?;

        let program = Program::from_source(display, vertex_source.as_str(), fragment_source.as_str(), None)?;

        Ok(Self {
            program
        })
    }
}