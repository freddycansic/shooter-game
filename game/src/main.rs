use winit::window::WindowAttributes;

use common::runtime::run;
use game::Game;

mod controllers;
mod game;

fn main() {
    run::run::<Game>(WindowAttributes::default());
}
