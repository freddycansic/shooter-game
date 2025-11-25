use std::path::Path;

use color_eyre::eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Frame, Surface};
use nalgebra::{Matrix4, Point3};
use rfd::FileDialog;

use crate::camera::FpsCamera;
use crate::colors::{Color, ColorExt};
use crate::light::Light;
use crate::line::Line;
use crate::renderer::Renderer;
use crate::resources::handle::CubemapHandle;
use crate::resources::resources::Resources;
use crate::scene::graph::{NodeType, Renderable, SceneGraph, SceneNode};
use crate::serde::SerializedScene;
use crate::transform::Transform;

#[derive(PartialEq, Clone)]
pub enum Background {
    Color(Color),
    HDRI(CubemapHandle),
}

impl Default for Background {
    fn default() -> Self {
        Background::Color(Color::from_named(palette::named::GRAY))
    }
}

pub struct Scene {
    pub title: String,
    pub camera: FpsCamera, // the camera state to be used when starting the game
    pub graph: SceneGraph,
    pub background: Background,
    pub lights: Vec<Light>,
    // pub terrain: Option<Terrain>,
    // pub quads: StableDiGraph<Quad, ()>,
    // #[serde(skip)]
    pub lines: Vec<Line>,
    pub resources: Resources,
}

impl Scene {
    pub fn new(title: &str) -> Self {
        Self {
            graph: SceneGraph::new(),
            lines: vec![],
            // quads: StableDiGraph::new(),
            title: title.to_owned(),
            camera: FpsCamera::default(),
            background: Background::default(),
            // terrain: None,
            lights: vec![],
            resources: Resources::new(),
        }
    }

    // pub fn from_path(path: &Path, display: &Display<WindowSurface>) -> Result<Self> {
    //     let serialized_scene_string = std::fs::read_to_string(path)?;

    //     let serialized_scene = serde_json::from_str::<SerializedScene>(&serialized_scene_string)?;

    //     serialized_scene.into_scene(display)
    // }

    // pub fn from_string(scene_string: &str, display: &Display<WindowSurface>) -> Result<Self> {
    //     let mut scene = serde_json::from_str::<Scene>(scene_string)?;

    //     let node_indices = scene.graph.node_indices().collect_vec();

    //     // Load assets which require Display
    //     for node_index in node_indices {
    //         // Cannot change call to unwrap to "?" because Mutex is not Send, and ErrReport must be Send
    //         // TODO
    //         // if scene.graph[node_index]
    //         //     .model
    //         //     .meshes
    //         //     .lock()
    //         //     .unwrap()
    //         //     .is_none()
    //         // {
    //         //     scene.graph[node_index].model.load_meshes(display).unwrap()
    //         // }

    //         // if let Some(material) = scene.graph[node_index].material.as_mut() {
    //         //     material.diffuse = Texture2D::load(material.diffuse.path.clone(), display)?;
    //         // }
    //     }

    //     // for (_, model_instance) in scene.graph.node_references() {
    //     //     if model_instance.model.meshes.lock().unwrap().is_none() {
    //     //         model_instance.model.load_meshes(display).unwrap()
    //     //     }
    //     //
    //     //     if let Some(material) = model_instance.material.as_mut() {
    //     //         material.diffuse = Texture2D::load(material.diffuse.path.clone(), display)?;
    //     //     }
    //     // }

    //     if let Background::HDRI(cubemap) = scene.background {
    //         scene.background = Background::HDRI(Cubemap::load(cubemap.directory.clone(), display)?);
    //     }

    //     for quad in scene.quads.node_weights_mut() {
    //         quad.texture = Texture2D::load(quad.texture.path.clone(), display)?;
    //     }

    //     Ok(scene)
    // }

    pub fn save_as(&self) {
        let serialized_scene = SerializedScene::from_scene(self);

        let serialized = serde_json::to_string(&serialized_scene).unwrap();

        std::thread::spawn(move || {
            if let Some(save_path) = FileDialog::new().save_file() {
                std::fs::write(save_path, serialized).unwrap();
            }
        });
    }

    /// Load a models and create an instance of it in the scene
    pub fn import_model(&mut self, path: &Path, display: &Display<WindowSurface>) -> Result<()> {
        let handles = self.resources.get_geometry_handles(path, display)?;

        let group_node = self
            .graph
            .add_root_node(SceneNode::new(NodeType::Group, Transform::identity()));

        let texture_handle = self
            .resources
            .get_texture_handle(Path::new("assets/textures/uv-test.jpg"), display)?;

        for geometry_handle in handles {
            let renderable = Renderable {
                geometry_handle,
                texture_handle,
            };

            let scene_node =
                SceneNode::new(NodeType::Renderable(renderable), Transform::identity());

            let node_index = self.graph.add_node(scene_node);

            self.graph.add_edge(group_node, node_index);
        }

        self.graph.add_edge(self.graph.root, group_node);

        Ok(())
    }

    pub fn render(
        &mut self,
        renderer: &mut Renderer,
        view: &Matrix4<f32>,
        camera_position: Point3<f32>,
        _debug: bool,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        match &self.background {
            Background::Color(color) => target.clear_all(color.to_rgb_components_tuple(), 1.0, 0),
            Background::HDRI(cubemap_handle) => {
                target.clear_all(
                    Color::from_named(palette::named::WHITE).to_rgb_components_tuple(),
                    1.0,
                    0,
                );
                renderer.render_skybox(*cubemap_handle, &self.resources, view, target);
            }
        }

        let queue = self.graph.build_render_queue();

        renderer.render_queue(
            queue,
            &self.resources,
            view,
            camera_position,
            &self.lights,
            display,
            target,
        );

        // if let Some(terrain) = &self.terrain {
        //     renderer.render_terrain(terrain, view, camera_position, target);
        // }

        // renderer.render_lines(&self.lines, view, display, target);

        // if debug {
        //     renderer.render_lights(&self.lights, view, display, target);
        // }

        // // Render quads last so they stay on top
        // renderer.render_quads(&self.quads, display, target);
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled")
    }
}
