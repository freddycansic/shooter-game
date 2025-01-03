use crate::camera::FpsCamera;
use crate::colors::{from_named, Color, ColorExt};
use crate::line::Line;
use crate::models::Model;
use crate::models::ModelInstance;
use crate::renderer::Renderer;
use crate::texture::Cubemap;
use cgmath::{Matrix4, Point3};
use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Frame, Surface};
use petgraph::prelude::StableDiGraph;
use petgraph::visit::IntoNodeReferences;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

#[derive(PartialEq, Serialize, Deserialize)]
pub enum Background {
    Color(Color),
    HDRI(Arc<Cubemap>),
}

impl Default for Background {
    fn default() -> Self {
        Background::Color(from_named(palette::named::GRAY))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub title: String,
    pub camera: FpsCamera, // the camera state to be used when starting the game
    pub graph: StableDiGraph<ModelInstance, ()>,
    pub background: Background,
    #[serde(skip)]
    pub lines: Vec<Line>,
}

impl Scene {
    pub fn new(title: &str) -> Self {
        Self {
            graph: StableDiGraph::new(),
            lines: vec![],
            title: title.to_owned(),
            camera: FpsCamera::default(),
            background: Background::default(),
        }
    }

    pub fn from_path(path: &Path, display: &Display<WindowSurface>) -> Result<Self> {
        Self::from_string(&std::fs::read_to_string(path)?, display)
    }

    pub fn from_string(scene_string: &str, display: &Display<WindowSurface>) -> Result<Self> {
        let mut scene = serde_json::from_str::<Scene>(scene_string)?;

        // Cannot change call to unwrap to "?" because Mutex is not Send, and ErrReport must be Send
        for (_, model_instance) in scene.graph.node_references() {
            if model_instance.model.meshes.lock().unwrap().is_none() {
                model_instance.model.load_meshes(display).unwrap()
            }
        }

        if let Background::HDRI(cubemap) = scene.background {
            scene.background = Background::HDRI(Cubemap::load(cubemap.directory.clone(), display)?);
        }

        Ok(scene)
    }

    pub fn save_as(&self) {
        let serialized = serde_json::to_string(self).unwrap();

        std::thread::spawn(move || {
            if let Some(save_path) = FileDialog::new().save_file() {
                std::fs::write(save_path, serialized).unwrap();
            }
        });
    }

    /// Load a models and create an instance of it in the scene
    pub fn import_model(&mut self, path: &Path, display: &Display<WindowSurface>) -> Result<()> {
        let model = Model::load(path.to_path_buf(), display)?;

        self.graph.add_node(ModelInstance::from(model));

        Ok(())
    }

    pub fn render(
        &mut self,
        renderer: &mut Renderer,
        view: &Matrix4<f32>,
        projection: &Matrix4<f32>,
        camera_position: Point3<f32>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        match &self.background {
            Background::Color(color) => {
                target.clear_color_and_depth(color.to_rgb_vector4().into(), 1.0)
            }
            Background::HDRI(cubemap) => {
                target.clear_color_and_depth(
                    from_named(palette::named::WHITE).to_rgb_vector4().into(),
                    1.0,
                );
                renderer.render_skybox(cubemap, view, projection, target);
            }
        }

        let view_projection = projection * view;

        renderer.render_model_instances(
            self.graph.node_references(),
            &view_projection,
            camera_position,
            display,
            target,
        );
        renderer.render_lines(&self.lines, &view_projection, display, target);
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled")
    }
}
