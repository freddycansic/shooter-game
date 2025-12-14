use winit::{platform::wayland::WindowAttributesExtWayland, window::WindowAttributes};

use common::run;
use editor::Editor;

mod editor;
mod ui;

fn main() {
    let attributes = WindowAttributes::default().with_name("shooter-game-editor", "");
    run::run::<Editor>(attributes);
}
