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
    pub scheduler: Scheduler,
}

impl Engine {
    pub fn new(
    ) -> Self {
        Self {
            scheduler: Scheduler::default(),
        }
    }
}
