mod app;
mod camera;
mod colors;
mod context;
mod debug;
mod input;
mod maths;
mod model;
mod scene;
mod vertex;
mod uuid;

use crate::app::App;
use winit::event_loop::EventLoop;

fn main() {
    // Without this, hyprland crashes randomly
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let app = App::new(&event_loop);
    app.run(event_loop);
}
