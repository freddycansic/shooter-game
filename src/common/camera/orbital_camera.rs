use crate::input::Input;

use crate::camera::camera;
use crate::camera::camera::Camera;
use cgmath::{Matrix4, Point3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OrbitalCamera {
    pub target: Point3<f32>,
    pub radius: f32,

    projection: Matrix4<f32>,
    position: Point3<f32>,
    yaw: f32,
    pitch: f32,
}

impl OrbitalCamera {
    fn new(target: Point3<f32>, radius: f32, ratio: f32) -> Self {
        Self {
            position: Point3::new(radius, 0.0, 0.0),
            radius,
            target,
            projection: camera::perspective(ratio),
            yaw: 0.0,
            pitch: std::f32::consts::FRAC_PI_2,
        }
    }

    pub fn update_zoom(&mut self, input: &Input) {
        let mouse_wheel_offset = input.mouse_wheel_offset();

        let zoom_step = 0.4;
        self.radius -= mouse_wheel_offset * zoom_step;

        self.update_position();
    }

    fn update_position(&mut self) {
        self.position = self.target
            + Vector3::new(
                self.radius * self.pitch.sin() * self.yaw.cos(),
                self.radius * self.pitch.cos(),
                self.radius * self.pitch.sin() * self.yaw.sin(),
            );
    }
}

impl Camera for OrbitalCamera {
    fn update(&mut self, input: &Input, deltatime: f32) {
        let sensitivity = 100.0;

        let offset = input.device_offset() * deltatime * sensitivity;

        self.yaw += offset.x;
        self.yaw %= 2.0 * std::f32::consts::PI;

        self.pitch -= offset.y;
        let epsilon = 0.000000001;
        self.pitch = self.pitch.clamp(epsilon, std::f32::consts::PI - epsilon);

        self.update_position();
    }

    fn set_aspect_ratio(&mut self, ratio: f32) {
        self.projection = camera::perspective(ratio);
    }

    fn position(&self) -> Point3<f32> {
        self.position
    }

    fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y())
    }

    fn projection(&self) -> Matrix4<f32> {
        self.projection
    }
}

impl Default for OrbitalCamera {
    fn default() -> Self {
        Self::new(Point3::new(0.0, 0.0, 0.0), 5.0, 1920.0 / 1080.0)
    }
}
