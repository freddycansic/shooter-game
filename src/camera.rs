use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};
use winit::keyboard::KeyCode;

use crate::input::Input;

pub enum ViewMode {
    FPS,
    Orbit
}

pub struct Camera {
    pub position: Point3<f32>,
    pub forward_direction: Vector3<f32>,
    pub up_direction: Vector3<f32>,
    pub target: Point3<f32>,
    pub projection: Matrix4<f32>,
    pub view_mode: ViewMode
}

impl Camera {
    pub fn new_fps(position: Point3<f32>, forward_direction: Vector3<f32>) -> Self {
        Self {
            position,
            target: position + forward_direction,
            forward_direction,
            up_direction: forward_direction.cross(Vector3::unit_y()).cross(forward_direction),
            projection: Self::create_perspective_matrix(1920.0 / 1080.0),
            view_mode: ViewMode::FPS
        }
    }

    pub fn new_orbital(position: Point3<f32>, target: Point3<f32>) -> Self {
        unimplemented!()
    }

    pub fn update(&mut self, input: &Input) {
        match self.view_mode {
            ViewMode::Orbit => unimplemented!(),
            ViewMode::FPS => self.update_fps(input)
        }
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.projection = Self::create_perspective_matrix(aspect_ratio);
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position, self.position + self.forward_direction, Vector3::new(0.0, -1.0, 0.0))
    }

    fn create_perspective_matrix(aspect_ratio: f32) -> Matrix4<f32> {
        cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0)
    }

    fn update_fps(&mut self, input: &Input) {
        let speed = 0.1;

        self.forward_direction = Vector3::<f32>::new(0.0, 0.0, 1.0);

        let right_direction = self.forward_direction.cross(Vector3::unit_y()).normalize();
        self.up_direction = self.forward_direction.cross(right_direction);

        if input.key_down(KeyCode::KeyW) {
            self.position += speed * self.forward_direction;
        }

        if input.key_down(KeyCode::KeyS) {
            self.position -= speed * self.forward_direction;
        }

        if input.key_down(KeyCode::KeyA) {
            self.position += speed * right_direction;
        }

        if input.key_down(KeyCode::KeyD) {
            self.position -= speed * right_direction;
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        // Camera at (0, 0, 0), looking down the z axis
        Self::new_fps(Point3::new(0.0, 0.0, 0.0), Vector3::unit_z())
    }
}
