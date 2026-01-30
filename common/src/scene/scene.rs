use std::mem::Discriminant;
use std::path::Path;

use color_eyre::eyre::Result;
use fxhash::FxHashMap;
use glium::glutin::surface::WindowSurface;
use glium::{Display, Frame, Surface};
use itertools::Itertools;
use nalgebra::{Matrix4, Point3, Vector3};
use petgraph::graph::NodeIndex;
use rfd::FileDialog;

use crate::camera::FpsCamera;
use crate::collision::collidable::{Intersectable, SweepHit};
use crate::collision::colliders::bvh::Bvh;
use crate::collision::colliders::sphere::Sphere;
use crate::colors::{Color, ColorExt};
use crate::components::component::Component;
use crate::light::Light;
use crate::line::Line;
use crate::maths::{Ray, Transform};
use crate::renderer::Renderer;
use crate::resources::CubemapHandle;
use crate::resources::Resources;
use crate::scene::graph::{GeometryBatches, NodeType, Renderable, SceneGraph, SceneNode};
use crate::scene::{QuadBatches, QuadTree};
use crate::serde::SerializedScene;

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

pub struct RenderQueue {
    pub geometry_batches: GeometryBatches,
    pub quad_batches: QuadBatches,
}

pub struct Scene {
    pub title: String,
    pub graph: SceneGraph,
    pub background: Background,
    pub lights: Vec<Light>,
    pub quads: QuadTree,
    pub lines: Vec<Line>,
    pub resources: Resources,
    pub terrain_bvh: Option<Bvh>,
}

impl Scene {
    pub fn new(title: &str) -> Self {
        Self {
            graph: SceneGraph::new(),
            lines: vec![],
            quads: QuadTree::new(),
            title: title.to_owned(),
            background: Background::default(),
            terrain_bvh: None,
            lights: vec![],
            resources: Resources::new(),
        }
    }

    pub fn intersect_ray(&mut self, ray: &Ray) -> Option<NodeIndex> {
        self.graph
            .graph
            .node_indices()
            .filter_map(|idx| {
                let node = &self.graph.graph[idx];
                node.intersect_ray(ray, &mut self.resources).map(|hit| (idx, hit))
            })
            .min_by(|(_, a), (_, b)| a.tmin.partial_cmp(&b.tmin).unwrap())
            .map(|(idx, _)| idx)
    }

    pub fn sweep_intersect_sphere(&mut self, sphere: &Sphere, velocity: &Vector3<f32>) -> Option<SweepHit> {
        self.graph
            .graph
            .node_indices()
            .filter_map(|idx| {
                let node = &self.graph.graph[idx];
                node.sweep_intersect_sphere(sphere, velocity, &mut self.resources)
            })
            .min_by(|a, b| a.partial_cmp(&b).unwrap())
    }

    // TODO: caching, rebuild on change component
    pub fn components(&self) -> FxHashMap<Discriminant<Component>, Vec<NodeIndex>> {
        let mut component_map = FxHashMap::<Discriminant<Component>, Vec<NodeIndex>>::default();

        let node_indices = self.graph.graph.node_indices().collect_vec();

        for node_index in node_indices.into_iter() {
            let node = &self.graph.graph[node_index];
            for component in &node.components {
                let node_vec = component_map.entry(std::mem::discriminant(component)).or_insert(vec![]);
                node_vec.push(node_index);
            }
        }

        component_map
    }

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
            .add_root_node(SceneNode::new(NodeType::Group));

        let texture_handle = self
            .resources
            .get_texture_handle(Path::new("assets/textures/uv-test.jpg"), display)?;

        for geometry_handle in handles {
            let renderable = Renderable {
                geometry_handle,
                texture_handle,
            };

            let scene_node = SceneNode::new(NodeType::Renderable(renderable));

            let node_index = self.graph.add_node(scene_node);

            self.graph.add_edge(group_node, node_index);
        }

        self.graph.add_edge(self.graph.root, group_node);

        Ok(())
    }

    pub fn build_render_queue(&mut self) -> RenderQueue {
        let geometry_batches = self.graph.batch_geometry();
        let quad_batches = self.quads.batch();

        RenderQueue {
            geometry_batches,
            quad_batches,
        }
    }

    pub fn render(
        &mut self,
        renderer: &mut Renderer,
        view: &Matrix4<f32>,
        camera_position: Point3<f32>,
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

        let queue = self.build_render_queue();

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

        renderer.render_lines(&self.lines, view, display, target);

        // if debug {
        //     renderer.render_lights(&self.lights, view, display, target);
        // }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled")
    }
}
