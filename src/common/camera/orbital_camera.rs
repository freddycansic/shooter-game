use crate::input::Input;

use crate::camera::camera;
use crate::camera::camera::Camera;
use cgmath::num_traits::Pow;
use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3, Zero};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OrbitalCamera {
    pub target: Point3<f32>,
    pub projection: Matrix4<f32>,
    pub position: Point3<f32>,
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
        let offset = input.device_offset();
        if offset.is_zero() {
            return;
        }

        let distance = (self.position - self.target).magnitude();
        dbg!(self.position);

        if distance == 0.0 {
            return;
        }

        // Consider the direction from the target to the camera
        let direction = (self.target - self.position).normalize();
        dbg!(direction);

        let sensitity = 50.0;

        dbg!(deltatime);
        let offset = input.device_offset() * deltatime * sensitity;
        dbg!(offset);

        let start_yaw = (direction.z / direction.x).atan();
        let start_pitch = direction.y.acos();
            // ((direction.x.powf(2.0) + direction.z.powf(2.0)).sqrt() / direction.z).atan();

        dbg!(start_yaw, start_pitch);
        // panic!();

        let new_yaw = (start_yaw + offset.x) % (2.0 * std::f32::consts::PI);
        let new_pitch =
            (start_pitch - offset.y).clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);
        dbg!(new_yaw, new_pitch);

        self.position = self.target
            + Vector3::new(
                distance * new_pitch.sin() * new_yaw.cos(),
                distance * new_pitch.cos(),
                distance * new_pitch.sin() * new_yaw.sin(),
            );

        dbg!(self.position);
        // panic!();
    }

    fn set_aspect_ratio(&mut self, ratio: f32) {
        self.projection = camera::perspective(ratio);
    }

    fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y())
    }
}

impl Default for OrbitalCamera {
    fn default() -> Self {
        Self::new(
            Point3::new(4.0, 4.0, 4.0),
            Point3::new(0.0, 0.0, 0.0),
            1920.0 / 1080.0,
        )
    }
}
