use std::fs;

use crate::application::Application;
use crate::executor::RuntimeContext;
use crate::subsystems::window_size::{WindowResized, WindowSize};
use color_eyre::Result;
use common::subsystems::frame_timing::WinitNewEvents;
use common_macros::Event;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Program, Vertex, VertexBuffer};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Debug)]
pub struct OpenGLContext<A: Application> {
    runtime: Option<Runtime<A>>,
    window_attributes: WindowAttributes,
}

#[derive(Debug)]
pub struct Runtime<A: Application> {
    pub window: Window,
    pub display: Display<WindowSurface>,
    pub application: A,
}

#[derive(Event)]
pub struct WinitWindowEvent(pub winit::event::WindowEvent);

#[derive(Event)]
pub struct WinitDeviceEvent(pub winit::event::DeviceEvent);

impl<'a, A: Application> OpenGLContext<A> {
    pub fn new(window_attributes: WindowAttributes) -> Self {
        Self {
            runtime: None,
            window_attributes,
        }
    }

    fn runtime(&mut self) -> &mut Runtime<A> {
        self.runtime.as_mut().expect("Runtime has not been created yet.")
    }
}

impl<'a, A: Application> ApplicationHandler for OpenGLContext<A> {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        self.runtime().new_events();
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = SimpleWindowBuilder::new()
            .set_window_builder(self.window_attributes.clone())
            .build(event_loop);

        self.runtime = Some(Runtime::new(window, display, event_loop));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        self.runtime().window_event(event_loop, window_id, event);
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        self.runtime().device_event(event_loop, device_id, event);
    }
}

impl<A: Application> Runtime<A> {
    pub fn new(mut window: Window, display: Display<WindowSurface>, event_loop: &ActiveEventLoop) -> Self {
        let context = RuntimeContext::new(&mut window, &display, event_loop);

        let mut application = A::new(&context);
        application.register_core_ecs_state(&context);

        Runtime {
            window,
            display,
            application,
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if window_id != self.window.id() {
            return;
        }

        self.application.world().write_event(WinitWindowEvent(event.clone()));

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                self.display.resize((new_size.width, new_size.height));

                self.application.world().write_event(WindowResized(WindowSize {
                    width: new_size.width,
                    height: new_size.height,
                }));
            }
            WindowEvent::RedrawRequested => {
                let context = RuntimeContext::new(&mut self.window, &self.display, event_loop);

                self.application.run(context);
            }
            _ => (),
        };

        self.application
            .window_event(event, event_loop, &self.window, &self.display);
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        self.application.world().write_event(WinitDeviceEvent(event.clone()));

        self.application
            .device_event(event, event_loop, &self.window, &self.display);
    }

    fn new_events(&mut self) {
        self.application.world().write_event(WinitNewEvents);
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
