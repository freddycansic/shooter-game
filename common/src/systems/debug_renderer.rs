use glium::{uniform, Blend, BlendingFunction, Depth, DepthTest, Display, DrawParameters, Frame, LinearBlendingFactor, Surface, VertexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::index::{NoIndices, PrimitiveType};
use itertools::Itertools;
use nalgebra::{Matrix4, Translation3};
use crate::camera::{Camera, OrbitalCamera};
use crate::colors::ColorExt;
use crate::debug::DebugCuboid;
use crate::maths;
use crate::systems::renderer::{RendererBuffers, SolidColorInstance};

pub struct DebugRenderer {
    pub cuboids: Vec<DebugCuboid>,

    debug_cube_instance_buffer: VertexBuffer<SolidColorInstance>,
}

impl DebugRenderer {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        Self {
            debug_cube_instance_buffer: VertexBuffer::empty(display, 10 /* ? */).unwrap(),
            cuboids: vec![],
        }
    }
}