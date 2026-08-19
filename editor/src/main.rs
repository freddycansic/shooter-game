use winit::window::WindowAttributes;

use editor::Editor;

mod editor;
mod ui;

fn main() {
    let mut attributes = WindowAttributes::default();

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            attributes = winit::platform::wayland::WindowAttributesExtWayland::with_name(attributes, "shooter-game-editor", "");
        }
    }

    common::runtime::run::<Editor>(attributes);
}
