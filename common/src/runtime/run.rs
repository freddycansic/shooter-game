use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowAttributes;

use crate::runtime::application::Application;
use crate::runtime::Runner;

pub fn run<A: Application>(window_attributes: WindowAttributes) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut opengl_context = Runner::<A>::new(window_attributes);
    event_loop.run_app(&mut opengl_context).unwrap()
}
