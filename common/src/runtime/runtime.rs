use crate::runtime::application::Application;
use common_macros::Event;
use glium::glutin::surface::WindowSurface;
use glium::Display;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::runtime::RuntimeContext;
use crate::subsystems::frame_timing::WinitNewEvents;
use crate::subsystems::window_size::{WindowResized, WindowSize};

#[derive(Event)]
pub struct WinitWindowEvent(pub winit::event::WindowEvent);

#[derive(Event)]
pub struct WinitDeviceEvent(pub winit::event::DeviceEvent);

#[derive(Debug)]
pub struct Runtime<A: Application> {
    pub window: Window,
    pub display: Display<WindowSurface>,
    pub application: A,
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

    pub fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
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

    pub fn device_event(&mut self, event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        self.application.world().write_event(WinitDeviceEvent(event.clone()));

        self.application
            .device_event(event, event_loop, &self.window, &self.display);
    }

    pub fn new_events(&mut self) {
        self.application.world().write_event(WinitNewEvents);
    }
}
