use winit::window::WindowAttributes;

use common::run;
use game::Game;

mod game;
mod controllers;

fn main() {
    run::run::<Game>(WindowAttributes::default());
}
