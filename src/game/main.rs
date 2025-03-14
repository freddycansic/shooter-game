use winit::window::WindowAttributes;

use common::application::Application;
use common::run;
use game::Game;

mod game;

fn main() {
    run::run::<Game>(WindowAttributes::default());
}
