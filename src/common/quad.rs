use crate::ui::UiItem;
use crate::{texture::Texture2D, ui};
use cgmath::{Point2, Vector2};
use egui_glium::egui_winit::egui::WidgetText;
use glium::implement_vertex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Quad {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub texture: Arc<Texture2D>,
    // Higher layer = closer to camera
    pub layer: i32,

    pub selected: bool,
    pub name: String,
}

impl Quad {
    pub fn new(
        position: Point2<f32>,
        size: Vector2<f32>,
        texture: Arc<Texture2D>,
        layer: i32,
    ) -> Self {
        Self {
            position,
            size,
            texture,
            layer,
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

impl From<Quad> for QuadVertex {
    fn from(value: Quad) -> Self {
        QuadVertex {
            position: <[f32; 2]>::from(value.position),
            size: <[f32; 2]>::from(value.size),
            layer: value.layer,
        }
    }
}

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
