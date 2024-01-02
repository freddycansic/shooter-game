#![feature(offset_of)]

mod app;
mod buffers;
mod debug;
mod maths;
mod model;
mod scene;
mod shaders;
mod texture;
mod vertex;

use crate::app::App;

fn main() {
    App::new().run();
}
