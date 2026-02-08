use egui_glium::EguiGlium;
use crate::input::Input;
use crate::systems::renderer::Renderer;
use crate::resources::Resources;
use crate::scene::graph::SceneGraph;
use crate::world::World;

pub struct Engine {
    pub renderer: Renderer,
    pub input: Input,
    pub resources: Resources,
    pub gui: EguiGlium,
}