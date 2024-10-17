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

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelInstance {
    pub model: Arc<Model>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<Arc<Texture>>,
    pub transform: Transform,
    #[serde(skip)]
    pub selected: bool,
}

impl From<Arc<Model>> for ModelInstance {
    fn from(model: Arc<Model>) -> Self {
        Self {
            model,
            name: "Model".to_owned(),
            texture: None,
            transform: Transform::default(),
            selected: false,
        }
    }
}
