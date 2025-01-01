use crate::input::Input;

use crate::camera::camera;
use crate::camera::camera::Camera;
use cgmath::{InnerSpace, Matrix4, Point3, Vector3};
use serde::{Deserialize, Serialize};
use winit::keyboard::KeyCode;

#[derive(Serialize, Deserialize)]
pub struct FpsCamera {
    projection: Matrix4<f32>,
    position: Point3<f32>,
    yaw: f32,
    pitch: f32,
    looking_direction: Vector3<f32>,
}

impl FpsCamera {
    fn new(position: Point3<f32>, ratio: f32) -> Self {
        Self {
            position,
            projection: camera::perspective(ratio),
            yaw: 0.0,
            pitch: std::f32::consts::FRAC_PI_2,
            looking_direction: Vector3::unit_x(),
        }
    }
}

impl Camera for FpsCamera {
    fn update(&mut self, input: &Input, deltatime: f32) {
        let mouse_sensitivity = 100.0;

        let offset = input.device_offset() * deltatime * mouse_sensitivity;

        self.yaw += offset.x;
        self.yaw %= 2.0 * std::f32::consts::PI;

        self.pitch -= offset.y;
        let epsilon = 0.00001;
        self.pitch = self.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + epsilon,
            std::f32::consts::FRAC_PI_2 - epsilon,
        );

        // No vertical movement
        self.looking_direction = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        let left_direction = self.looking_direction.cross(Vector3::unit_y());
        let forward_direction =
            Vector3::new(self.looking_direction.x, 0.0, self.looking_direction.z).normalize();

        let speed = 3.0;

        if input.key_down(KeyCode::KeyW) {
            self.position += forward_direction * deltatime * speed;
        }

        if input.key_down(KeyCode::KeyS) {
            self.position -= forward_direction * deltatime * speed;
        }

        if input.key_down(KeyCode::KeyA) {
            self.position -= left_direction * deltatime * speed;
        }

        if input.key_down(KeyCode::KeyD) {
            self.position += left_direction * deltatime * speed;
        }
    }

    fn set_aspect_ratio(&mut self, ratio: f32) {
        self.projection = camera::perspective(ratio);
    }

    fn position(&self) -> Point3<f32> {
        self.position
    }

    fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            self.position,
            self.position + self.looking_direction,
            Vector3::unit_y(),
        )
    }

    fn projection(&self) -> Matrix4<f32> {
        self.projection
    }
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self::new(Point3::new(0.0, 0.0, 0.0), 1920.0 / 1080.0)
    }
}
