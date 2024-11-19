use crate::input::Input;
use cgmath::{Matrix4, Rad};

pub trait Camera {
    fn update(&mut self, input: &Input, deltatime: f32);
    fn update_perspective(&mut self, ratio: f32);

    fn view(&self) -> Matrix4<f32>;
    fn projection(&self) -> Matrix4<f32>;
}

pub fn perspective(ratio: f32) -> Matrix4<f32> {
    cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), ratio, 0.01, 100.0)
}
