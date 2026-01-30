mod ui_graph;
mod ui_transform;
pub mod components;

use egui_glium::egui_winit::egui::Ui;

pub trait Show {
    fn show(&mut self, ui: &mut Ui);
}
