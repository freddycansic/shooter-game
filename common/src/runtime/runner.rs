use std::fs;

use crate::runtime::application::Application;
use crate::runtime::Runtime;
use crate::subsystems::window_size::{WindowResized, WindowSize};
use color_eyre::Result;
use common_macros::Event;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Program, Vertex, VertexBuffer};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Debug)]
pub struct Runner<A: Application> {
    runtime: Option<Runtime<A>>,
    window_attributes: WindowAttributes,
}

impl<'a, A: Application> Runner<A> {
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

impl<'a, A: Application> ApplicationHandler for Runner<A> {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        match &mut self.runtime {
            Some(runtime) => {
                runtime.new_events();
            }
            None => log::warn!("Received new_events before runtime initialisation."),
        }
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
