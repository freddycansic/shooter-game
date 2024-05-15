use bytemuck::{Pod, Zeroable};
use cgmath::Point3;
use glium::implement_vertex;
use log::warn;
use palette::Srgb;

#[derive(Clone)]
pub struct Line {
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub color: Srgb,
    pub width: u8,
}

impl Line {
    pub fn new(p1: Point3<f32>, p2: Point3<f32>, color: Srgb, width: u8) -> Self {
        if width > 10 {
            warn!("Line width can only be integer values between 1 and 10.");
        }

        Self {
            p1,
            p2,
            color,
            width,
        }
    }
}

#[derive(Copy, Clone)]
pub struct LinePoint {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

implement_vertex!(LinePoint, position, color);
