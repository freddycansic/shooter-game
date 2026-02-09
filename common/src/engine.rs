use crate::input::Input;
use crate::resources::Resources;
use crate::systems::renderer::Renderer;
use egui_glium::EguiGlium;

pub struct Engine {
    pub renderer: Renderer,
    pub input: Input,
    pub resources: Resources,
    pub gui: EguiGlium,
}
