mod app;
mod buffers;
mod camera;
mod colors;
mod context;
mod debug;
mod maths;
mod model;
mod pipeline;
mod scene;
mod shaders;
mod texture;
mod vertex;

use crate::app::App;
use winit::event_loop::EventLoop;

fn main() {
    // Without this, hyprland crashes randomly
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    // God level variable
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    let app = App::new(&event_loop);
    app.run(event_loop);
}
