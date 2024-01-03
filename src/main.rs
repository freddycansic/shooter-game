#![feature(offset_of)]

mod app;
mod buffers;
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
    // God level variable
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    let app = App::new(&event_loop);
    app.run(event_loop);
}
