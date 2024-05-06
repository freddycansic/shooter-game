mod editor;
use winit::event_loop::EventLoop;
use common::app::Application;
use editor::Editor;

fn main() {
    // Winit is dodgey on Wayland, prefer to use Xwayland
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let editor = Editor::new(&event_loop);
    editor.run(event_loop);
}
