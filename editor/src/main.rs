use winit::window::WindowAttributes;

use common::run;
use editor::Editor;

mod editor;
mod ui;

fn main() {
    let mut attributes = WindowAttributes::default();

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            platform::wayland::WindowAttributesExtWayland::with_name(attributes, "shooter-game-editor", "");
        }
    }

    run::run::<Editor>(attributes);
}
