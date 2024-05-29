use crate::model::Model;
use crate::texture::Texture;
use crate::transform::Transform;
use crate::{model, texture};
use color_eyre::Result;
use egui_glium::egui_winit::egui::TextBuffer;
use glium::glutin::surface::WindowSurface;
use glium::Display;
use serde::de::{EnumAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct ModelInstance {
    pub model: Arc<Model>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<Arc<Texture>>,
    pub transform: Transform,
}

impl From<Arc<Model>> for ModelInstance {
    fn from(model: Arc<Model>) -> Self {
        Self {
            model,
            texture: None,
            transform: Transform::default(),
        }
    }
}
