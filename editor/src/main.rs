use winit::window::WindowAttributes;

use common::run;
use editor::Editor;

mod editor;

fn main() {
    run::run::<Editor>(WindowAttributes::default());
}
