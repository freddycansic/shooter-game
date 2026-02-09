use winit::window::WindowAttributes;

use common::run;
use game::Game;

mod controllers;
mod game;

fn main() {
    run::run::<Game>(WindowAttributes::default());
}
