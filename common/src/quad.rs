use crate::resources::TextureHandle;
use crate::ui;
use crate::ui::UiItem;
use egui_glium::egui_winit::egui::WidgetText;
use glium::implement_vertex;
use nalgebra::{Point2, Vector2};

// #[derive(Clone, Serialize, Deserialize)]
pub struct Quad {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub texture: TextureHandle,

    pub selected: bool,
    pub name: String,
}

impl Quad {
    pub fn new(position: Point2<f32>, size: Vector2<f32>, texture: TextureHandle) -> Self {
        Self {
            position,
            size,
            texture,
            selected: false,
            name: ui::default_name::quad(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct QuadVertex {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub layer: i32,
}
implement_vertex!(QuadVertex, position, size, layer);

impl UiItem for Quad {
    fn name(&self) -> WidgetText {
        (&self.name).into()
    }

    // TODO make it proc macro if this becomes a problem
    fn selected(&self) -> bool {
        self.selected
    }

    fn select(&mut self) {
        self.selected = true;
    }

    fn deselect(&mut self) {
        self.selected = false;
    }
}
