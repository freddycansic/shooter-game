mod game;

use common::app::Application;
use game::Game;
use winit::event_loop::EventLoop;

fn main() {
    // Winit is dodgey on Wayland, prefer to use Xwayland
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let game = Game::new(&event_loop);
    game.run(event_loop);
}
