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
use crate::systems::renderer::{Renderable, Renderer};
use crate::resources::CubemapHandle;
use crate::resources::Resources;
use crate::scene::graph::{SceneGraph, SceneNode};
use crate::scene::{QuadBatches, QuadTree};
use crate::serde::SerializedScene;

pub struct Scene {
    pub title: String,
    pub graph: SceneGraph,
}

impl Scene {
    pub fn new(title: &str) -> Self {
        Self {
            graph: SceneGraph::new(),
            title: title.to_owned(),
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled")
    }
}
