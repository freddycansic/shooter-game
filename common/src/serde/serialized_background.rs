use std::path::PathBuf;

use glium::{Display, glutin::surface::WindowSurface};
use serde::{Deserialize, Serialize};

use crate::systems::renderer::Background;
use crate::{colors::Color, resources::Resources};

#[derive(Serialize, Deserialize)]
pub enum SerializedBackground {
    Color(Color),
    HDRI(PathBuf),
}

impl SerializedBackground {
    pub fn from_background(background: &Background, resources: &Resources) -> Self {
        match background {
            Background::Color(color) => SerializedBackground::Color(color.clone()),
            Background::HDRI(hdri_handle) => SerializedBackground::HDRI(resources.get_cubemap_path(*hdri_handle)),
        }
    }

    pub fn into_background(self, display: &Display<WindowSurface>, resources: &mut Resources) -> Background {
        match self {
            SerializedBackground::Color(color) => Background::Color(color.clone()),
            SerializedBackground::HDRI(hdri_path) => {
                Background::HDRI(resources.get_cubemap_handle(&hdri_path, display).unwrap())
            }
        }
    }
}
