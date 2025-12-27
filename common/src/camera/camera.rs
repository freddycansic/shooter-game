use nalgebra::{Matrix4, Point3, Vector3};

use crate::input::Input;

pub trait Camera {
    fn update(&mut self, input: &Input, deltatime: f32);

    fn position(&self) -> Point3<f32>;
    fn view(&self) -> Matrix4<f32>;
    fn direction(&self) -> Vector3<f32>;
}
