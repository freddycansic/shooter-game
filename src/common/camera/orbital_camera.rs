use crate::input::Input;

use crate::camera::camera;
use crate::camera::camera::Camera;
use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};

pub struct OrbitalCamera {
    pub target: Point3<f32>,

    position: Point3<f32>,
    projection: Matrix4<f32>,
}

impl OrbitalCamera {
    fn new(position: Point3<f32>, target: Point3<f32>, ratio: f32) -> Self {
        Self {
            position,
            target,
            projection: camera::perspective(ratio),
        }
    }
}

impl Camera for OrbitalCamera {
    fn update(&mut self, input: &Input, deltatime: f32) {
        let distance = (self.position - self.target).magnitude();

        // Consider the direction from the target to the camera
        let inverse_direction = (self.position - self.target).normalize();

        let offset = input.device_offset();
        let yaw = (inverse_direction.y / inverse_direction.x).atan()
            + offset.x % (2.0 * std::f32::consts::PI);
        let pitch = inverse_direction.z.acos() - offset.y;

        let new_inverse_direction = Vector3::new(
            distance * pitch.sin() * yaw.cos(),
            distance * pitch.sin() * yaw.sin(),
            distance * pitch.cos(),
        );

        self.position = self.target + new_inverse_direction;
    }

    fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y())
    }

    fn projection(&self) -> Matrix4<f32> {
        self.projection
    }

    fn update_perspective(&mut self, ratio: f32) {
        self.projection = camera::perspective(ratio);
    }
}

impl Default for OrbitalCamera {
    fn default() -> Self {
        Self::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            1920.0 / 1080.0,
        )
    }
}
