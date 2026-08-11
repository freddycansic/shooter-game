use crate::ecs::system_parameters::commands::Commands;
use crate::ecs::system_parameters::res::Res;
use crate::engine::input::{Input, InputReceived};
use crate::engine::scheduler::Stage;
use crate::executor::RuntimeContext;
use crate::maths;
use crate::subsystems::window_size::{WindowResized, WindowSize};
use common::ecs::subsystem::Subsystem;
use common::ecs::system_parameters::res::ResMut;
use common::engine::scheduler::Scheduler;
use common::executor::CommandExecutor;
use common::subsystems::frame_timing::FrameTiming;
use common::world::World;
use common_macros::Resource;
use nalgebra::{Matrix4, Point3, Vector3};
use serde::{Deserialize, Serialize};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Resource, Serialize, Deserialize)]
pub struct OrbitalCamera {
    pub target: Point3<f32>,
    pub radius: f32,

    orthographic_projection: Matrix4<f32>,
    perspective_projection: Matrix4<f32>,
    view: Matrix4<f32>,
    vp: Matrix4<f32>,
    inv_vp: Matrix4<f32>,

    position: Point3<f32>,
    yaw: f32,
    pitch: f32,
}

impl OrbitalCamera {
    pub fn new(target: Point3<f32>, radius: f32, width: f32, height: f32) -> Self {
        let perspective_projection = maths::perspective_matrix_from_dimensions(width, height);
        let orthographic_projection = maths::orthographic_matrix_from_dimensions(width, height);

        let position = Point3::new(radius, 0.0, 0.0);
        let view = Self::calculate_view(&position, &target);
        let vp = view * perspective_projection;
        let inv_vp = vp.try_inverse().unwrap();

        Self {
            position,
            radius,
            target,
            yaw: 0.0,
            pitch: std::f32::consts::FRAC_PI_2,
            perspective_projection,
            orthographic_projection,
            view,
            vp,
            inv_vp,
        }
    }

    pub fn perspective_projection(&self) -> Matrix4<f32> {
        self.perspective_projection
    }

    pub fn orthographic_projection(&self) -> Matrix4<f32> {
        self.orthographic_projection
    }

    pub fn update_projection_matrices(&mut self, width: f32, height: f32) {
        self.perspective_projection = maths::perspective_matrix_from_dimensions(width, height);
        self.orthographic_projection = maths::orthographic_matrix_from_dimensions(width, height);
    }

    pub fn position(&self) -> Point3<f32> {
        self.position
    }

    pub fn view(&self) -> Matrix4<f32> {
        self.view
    }

    pub fn vp(&self) -> Matrix4<f32> {
        self.vp
    }

    pub fn inv_vp(&self) -> Matrix4<f32> {
        self.inv_vp
    }

    pub fn direction(&self) -> Vector3<f32> {
        (self.target - self.position).normalize()
    }

    fn update_position(&mut self) {
        self.position = self.target
            + Vector3::new(
                self.radius * self.pitch.sin() * self.yaw.cos(),
                self.radius * self.pitch.cos(),
                self.radius * self.pitch.sin() * self.yaw.sin(),
            );

        self.update_view_matrices();
    }

    fn calculate_view(position: &Point3<f32>, target: &Point3<f32>) -> Matrix4<f32> {
        Matrix4::look_at_rh(position, target, &Vector3::new(0.0, 1.0, 0.0))
    }

    fn update_view_matrices(&mut self) {
        self.view = Self::calculate_view(&self.position, &self.target);
        self.vp = self.view * self.perspective_projection;
        self.inv_vp = self.vp.try_inverse().unwrap();
    }
}

fn update_zoom(mut camera: ResMut<OrbitalCamera>, input: Res<Input>) {
    let mouse_wheel_offset = input.mouse_wheel_offset();

    let zoom_step = 0.4;
    camera.radius -= mouse_wheel_offset * zoom_step;

    camera.update_position();
}

fn update_angles(
    mut camera: ResMut<OrbitalCamera>,
    frame_state: Res<FrameTiming>,
    input: Res<Input>,
    mut engine_commands: Commands,
) {
    let can_move_camera = input.mouse_button_down(MouseButton::Middle) || input.key_down(KeyCode::Space);

    if can_move_camera {
        let sensitivity = 200.0;

        let offset = input.device_offset() * frame_state.deltatime as f32 * sensitivity;

        camera.yaw += offset.x;
        camera.yaw %= 2.0 * std::f32::consts::PI;

        camera.pitch -= offset.y;
        camera.pitch = camera.pitch.clamp(f32::EPSILON, std::f32::consts::PI - f32::EPSILON);

        camera.update_position();

        engine_commands.capture_cursor();
        engine_commands.center_cursor();
    } else {
        engine_commands.release_cursor();
    }
}

fn window_resized(mut camera: ResMut<OrbitalCamera>, window_size: Res<WindowSize>) {
    camera.update_projection_matrices(window_size.width as f32, window_size.height as f32);
}

impl Subsystem for OrbitalCamera {
    fn register_resources(world: &mut World, context: Option<&RuntimeContext>) {
        let camera_radius = 5.0;
        let window_size = context.unwrap().window.inner_size();

        world.register_resource(OrbitalCamera::new(
            Point3::origin(),
            camera_radius,
            window_size.width as f32,
            window_size.height as f32,
        ));
    }

    fn register_systems(scheduler: &mut Scheduler) {
        scheduler.register_triggered::<InputReceived, _, _>(update_zoom, Stage::Main);
        scheduler.register_triggered::<InputReceived, _, _>(update_angles, Stage::Main);
        scheduler.register_triggered::<WindowResized, _, _>(window_resized, Stage::Main);
    }
}
