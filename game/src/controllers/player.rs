use common::input::Input;
use nalgebra::Vector3;
use petgraph::prelude::NodeIndex;
use winit::keyboard::KeyCode;

pub struct PlayerController {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub node: NodeIndex,
}

impl PlayerController {
    pub fn intended_velocity(&mut self, input: &Input, deltatime: f32) -> Vector3<f32> {
        // TODO
        let forward_direction = Vector3::z_axis().into_inner();
        let left_direction = Vector3::x_axis().into_inner();
        let speed = 1.0;

        let mut intended_velocity = Vector3::new(0.0, 0.0, 0.0);

        if input.key_down(KeyCode::KeyW) {
            intended_velocity += forward_direction * deltatime * speed;
        }

        if input.key_down(KeyCode::KeyS) {
            intended_velocity -= forward_direction * deltatime * speed;
        }

        if input.key_down(KeyCode::KeyA) {
            intended_velocity -= left_direction * deltatime * speed;
        }

        if input.key_down(KeyCode::KeyD) {
            intended_velocity += left_direction * deltatime * speed;
        }

        intended_velocity
    }
}
