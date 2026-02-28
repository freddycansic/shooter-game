use crate::engine::input::Input;
use crate::engine::renderer::Renderer;
use crate::engine::resources::Resources;
use egui_glium::EguiGlium;
use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::ViewportId;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use std::alloc::dealloc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct Engine {
    pub renderer: Renderer,
    pub input: Input,
    pub resources: Resources,
    pub gui: EguiGlium,
}

impl Engine {
    pub fn new(
        viewport: Option<egui::Rect>,
        display: &Display<WindowSurface>,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) -> Self {
        let mut resources = Resources::new();
        resources.initialise_default_texture(display).unwrap();

        Self {
            renderer: Renderer::new(viewport, display).unwrap(),
            input: Input::new(),
            resources,
            gui: EguiGlium::new(ViewportId::ROOT, display, window, event_loop),
        }
    }
}
