use crate::engine::assets::Assets;
use crate::engine::renderer::Renderer;
use common::engine::scheduler::Scheduler;
use egui_glium::EguiGlium;
use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::ViewportId;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct Engine {
    pub renderer: Renderer,
    pub assets: Assets,
    pub gui: EguiGlium,
    pub scheduler: Scheduler,
}

impl Engine {
    pub fn new(
        viewport: Option<egui::Rect>,
        display: &Display<WindowSurface>,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) -> Self {
        let mut assets = Assets::new();
        assets.initialise_default_texture(display).unwrap();

        Self {
            renderer: Renderer::new(viewport, display).unwrap(),
            assets,
            gui: EguiGlium::new(ViewportId::ROOT, display, window, event_loop),
            scheduler: Scheduler::default(),
        }
    }
}
