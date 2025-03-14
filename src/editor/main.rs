use winit::window::WindowAttributes;

use common::application::Application;
use common::run;
use editor::Editor;

mod editor;

fn main() {
    run::run::<Editor>(WindowAttributes::default());
}
