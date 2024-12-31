use std::fs;
use std::path::Path;

use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Frame, Surface};
use petgraph::prelude::StableDiGraph;
use petgraph::visit::IntoNodeReferences;
use rfd::FileDialog;
use serde::de::{MapAccess, Visitor};
use serde::ser::{SerializeMap, SerializeStruct, SerializeTuple};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::camera::camera::Camera;
use crate::camera::{FpsCamera, OrbitalCamera};
use crate::line::Line;
use crate::model::Model;
use crate::model_instance::ModelInstance;
use crate::renderer::Renderer;

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub camera: OrbitalCamera, // the last camera state when editing the scene
    pub title: String,
    pub starting_camera: FpsCamera, // the camera state to be used when starting the game
    pub graph: StableDiGraph<ModelInstance, ()>,
    #[serde(skip)]
    pub lines: Vec<Line>,
}

impl Scene {
    pub fn new(title: &str, camera: OrbitalCamera) -> Self {
        Self {
            graph: StableDiGraph::new(),
            lines: vec![],
            title: title.to_owned(),
            starting_camera: FpsCamera {},
            camera,
        }
    }

    pub fn from_path(path: &Path, display: &Display<WindowSurface>) -> Result<Self> {
        Self::from_string(&fs::read_to_string(path)?, display)
    }

    pub fn from_string(scene_string: &str, display: &Display<WindowSurface>) -> Result<Self> {
        let mut scene = serde_json::from_str::<Scene>(scene_string)?;

        for (_, model_instance) in scene.graph.node_references() {
            if model_instance.model.meshes.lock().unwrap().is_none() {
                model_instance.model.load_meshes(display).unwrap()
            }
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

    /// Load a model and create an instance of it in the scene
    pub fn import_model(&mut self, path: &Path, display: &Display<WindowSurface>) -> Result<()> {
        let model = Model::load(path.to_path_buf(), display)?;

        self.graph.add_node(ModelInstance::from(model));

        Ok(())
    }

    pub fn render(
        &mut self,
        renderer: &mut Renderer,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        target.clear_color_and_depth((0.01, 0.01, 0.01, 1.0), 1.0);

        let vp = self.camera.projection * self.camera.view();

        renderer.render_model_instances(
            self.graph.node_references(),
            &vp,
            self.camera.position(),
            display,
            target,
        );
        renderer.render_lines(&self.lines, &vp, display, target);
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled", OrbitalCamera::default())
    }
}
