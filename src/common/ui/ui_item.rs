use egui_glium::egui_winit::egui::WidgetText;

pub trait UiItem {
    fn name(&self) -> WidgetText;

    fn selected(&self) -> bool;
    fn select(&mut self);
    fn deselect(&mut self);

    fn toggle_selected(&mut self) {
        if self.selected() {
            self.deselect();
        } else {
            self.select();
        }
    }
}
