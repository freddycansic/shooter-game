use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowAttributes;

use crate::application::Application;
use crate::context::OpenGLContext;

pub fn run<A: Application>(window_attributes: WindowAttributes) {
    // Winit is dodgey on Wayland, prefer to use Xwayland
    // unsafe {
    //     std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    // }

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut opengl_context = OpenGLContext::<A>::new(window_attributes);
    event_loop.run_app(&mut opengl_context).unwrap()
}
