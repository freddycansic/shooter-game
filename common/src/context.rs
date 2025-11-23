use std::fs;

use color_eyre::Result;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Program, Vertex, VertexBuffer};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::application::Application;

#[derive(Debug)]
pub struct OpenGLContext<A: Application> {
    pub window: Option<Window>,
    pub display: Option<Display<WindowSurface>>,
    pub application: Option<A>,
    window_attributes: WindowAttributes,
}

impl<A: Application> OpenGLContext<A> {
    pub fn new(window_attributes: WindowAttributes) -> Self {
        Self {
            window: None,
            display: None,
            application: None,
            window_attributes,
        }
    }
}

impl<A: Application> ApplicationHandler for OpenGLContext<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = SimpleWindowBuilder::new()
            .set_window_builder(self.window_attributes.clone())
            .build(event_loop);

        self.window = Some(window);
        self.display = Some(display);

        self.application = Some(A::new(
            self.window.as_ref().unwrap(),
            self.display.as_ref().unwrap(),
            event_loop,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if window_id != self.window.as_ref().unwrap().id() {
            return;
        }

        self.application.as_mut().unwrap().window_event(
            event,
            event_loop,
            self.window.as_ref().unwrap(),
            self.display.as_ref().unwrap(),
        );
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.application.as_mut().unwrap().device_event(
            event,
            event_loop,
            self.window.as_ref().unwrap(),
            self.display.as_ref().unwrap(),
        );
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

pub fn new_sized_dynamic_vertex_buffer_with_data<T: Copy + Vertex>(
    display: &Display<WindowSurface>,
    size: usize,
    data: &[T],
) -> Result<VertexBuffer<T>> {
    assert!(size >= data.len());

    let mut vertex_buffer = VertexBuffer::<T>::empty_dynamic(display, size)?;

    vertex_buffer.slice_mut(..data.len()).unwrap().write(data);

    Ok(vertex_buffer)
}
