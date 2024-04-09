use egui::{Context, InputState};
use vulkano::buffer::{allocator::SubbufferAllocator, Subbuffer};

use cgmath::{EuclideanSpace, Matrix4, Point3, Rad, Vector3};
use winit::keyboard::KeyCode;

use crate::input::Input;
use crate::shaders::vs;
use crate::{buffers, input};

pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub projection: Matrix4<f32>,
}

impl Camera {
    pub fn new(position: Point3<f32>, target: Point3<f32>) -> Self {
        Self {
            position,
            target,
            projection: Self::create_perspective_matrix(1920.0 / 1080.0),
        }
    }

    pub fn update(&mut self, input: &Input) {
        let speed = 0.1;

        let forward_direction = self.target - self.position;

        if input.key_down(KeyCode::KeyW) {
            self.position += speed * forward_direction;
        }

        if input.key_down(KeyCode::KeyS) {
            self.position -= speed * forward_direction;
        }
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.projection = Self::create_perspective_matrix(aspect_ratio);
    }

    pub fn create_subbuffer(
        &self,
        subbuffer_allocator: &SubbufferAllocator,
    ) -> Subbuffer<vs::CameraUniform> {
        let view = Matrix4::look_at_rh(self.position, self.target, Vector3::new(0.0, -1.0, 0.0));

        let camera_uniform_data = vs::CameraUniform {
            view,
            projection: self.projection,
            camera_position: self.position.to_vec(),
        };

        buffers::create_subbuffer(subbuffer_allocator, camera_uniform_data)
    }

    fn create_perspective_matrix(aspect_ratio: f32) -> Matrix4<f32> {
        cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0)
    }
}

impl Default for Camera {
    fn default() -> Self {
        // Camera at (0, 0, 1), looking at the origin
        Self::new(Point3::new(0.0, 0.0, 1.0), Point3::new(0.0, 0.0, 0.0))
    }
}
