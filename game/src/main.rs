use winit::window::WindowAttributes;

use common::run;
use game::Game;

mod game;

fn main() {
    run::run::<Game>(WindowAttributes::default());
}
