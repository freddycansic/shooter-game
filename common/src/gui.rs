use common_macros::Resource;

// TODO move gui state into its own resource
#[derive(Resource)]
pub struct GuiState {
    pub render_lights: bool,
    pub debug_cube_index: usize,
    pub debug_cube_opacity: f32,
    pub render_debug_mouse_rays: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        GuiState {
            render_lights: true,
            debug_cube_index: 0,
            debug_cube_opacity: 0.5,
            render_debug_mouse_rays: false,
        }
    }
}
