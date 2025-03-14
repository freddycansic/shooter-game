use cgmath::{Matrix4, Point3, Rad};

use crate::input::Input;

pub trait Camera {
    fn update(&mut self, input: &Input, deltatime: f32);
    fn set_aspect_ratio(&mut self, ratio: f32);

    fn position(&self) -> Point3<f32>;
    fn projection(&self) -> Matrix4<f32>;
    fn view(&self) -> Matrix4<f32>;
}

pub fn perspective(ratio: f32) -> Matrix4<f32> {
    cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), ratio, 0.01, 100.0)
}
