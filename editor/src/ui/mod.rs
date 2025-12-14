mod ui_graph;

use egui_glium::egui_winit::egui::Ui;

pub trait Show {
    fn show(&mut self, ui: &mut Ui);
}
