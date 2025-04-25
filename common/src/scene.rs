use std::path::Path;
use std::sync::Arc;

use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Frame, Surface};
use itertools::Itertools;
use petgraph::prelude::StableDiGraph;
use rapier3d::na::{Matrix4, Point3};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use crate::camera::FpsCamera;
use crate::colors::{Color, ColorExt};
use crate::light::Light;
use crate::line::Line;
use crate::models::Model;
use crate::models::ModelInstance;
use crate::quad::Quad;
use crate::renderer::Renderer;
use crate::terrain::Terrain;
use crate::texture::{Cubemap, Texture2D};
use crate::physics::Physics;

#[derive(PartialEq, Serialize, Deserialize)]
pub enum Background {
    Color(Color),
    HDRI(Arc<Cubemap>),
}

impl Default for Background {
    fn default() -> Self {
        Background::Color(Color::from_named(palette::named::GRAY))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub title: String,
    pub camera: FpsCamera, // the camera state to be used when starting the game
    pub graph: StableDiGraph<ModelInstance, ()>,
    pub background: Background,
    pub lights: Vec<Light>,
    pub terrain: Option<Terrain>,
    pub quads: StableDiGraph<Quad, ()>,
    #[serde(skip)]
    pub lines: Vec<Line>,
    pub physics: Physics,
}

impl Scene {
    pub fn new(title: &str) -> Self {
        Self {
            graph: StableDiGraph::new(),
            lines: vec![],
            quads: StableDiGraph::new(),
            title: title.to_owned(),
            camera: FpsCamera::default(),
            background: Background::default(),
            terrain: None,
            lights: vec![],
            physics: Physics::default()
        }
    }

    pub fn from_path(path: &Path, display: &Display<WindowSurface>) -> Result<Self> {
        Self::from_string(&std::fs::read_to_string(path)?, display)
    }

    pub fn from_string(scene_string: &str, display: &Display<WindowSurface>) -> Result<Self> {
        let mut scene = serde_json::from_str::<Scene>(scene_string)?;

        let node_indices = scene.graph.node_indices().collect_vec();

        // Load assets which require Display
        for node_index in node_indices {
            // Cannot change call to unwrap to "?" because Mutex is not Send, and ErrReport must be Send
            if scene.graph[node_index]
                .model
                .meshes
                .lock()
                .unwrap()
                .is_none()
            {
                scene.graph[node_index].model.load_meshes(display).unwrap()
            }

            if let Some(material) = scene.graph[node_index].material.as_mut() {
                material.diffuse = Texture2D::load(material.diffuse.path.clone(), display)?;
            }
        }

        // for (_, model_instance) in scene.graph.node_references() {
        //     if model_instance.model.meshes.lock().unwrap().is_none() {
        //         model_instance.model.load_meshes(display).unwrap()
        //     }
        //
        //     if let Some(material) = model_instance.material.as_mut() {
        //         material.diffuse = Texture2D::load(material.diffuse.path.clone(), display)?;
        //     }
        // }

        if let Background::HDRI(cubemap) = scene.background {
            scene.background = Background::HDRI(Cubemap::load(cubemap.directory.clone(), display)?);
        }

        for quad in scene.quads.node_weights_mut() {
            quad.texture = Texture2D::load(quad.texture.path.clone(), display)?;
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
        camera_position: Point3<f32>,
        debug: bool,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        match &self.background {
            Background::Color(color) => target.clear_all(color.to_rgb_components_tuple(), 1.0, 0),
            Background::HDRI(cubemap) => {
                target.clear_all(
                    Color::from_named(palette::named::WHITE).to_rgb_components_tuple(),
                    1.0,
                    0,
                );
                renderer.render_skybox(cubemap, view, target);
            }
        }

        for model_instance in self.graph.node_weights_mut() {
            model_instance.transform.compute_transform_matrix();
        }

        renderer.render_model_instances(
            &self.graph,
            view,
            camera_position,
            &self.lights,
            display,
            target,
        );

        if let Some(terrain) = &self.terrain {
            renderer.render_terrain(terrain, view, camera_position, target);
        }

        renderer.render_lines(&self.lines, view, display, target);

        if debug {
            renderer.render_lights(&self.lights, view, display, target);
        }

        // Render quads last so they stay on top
        renderer.render_quads(&self.quads, display, target);
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled")
    }
}
