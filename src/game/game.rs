use std::path::PathBuf;
use std::time::Instant;

use glium::glutin::surface::WindowSurface;
use glium::Display;
use rapier3d::na::{Point2, Vector2};
use winit::event::{DeviceEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use common::application::Application;
use common::camera::Camera;
use common::debug;
use common::input::Input;
use common::quad::Quad;
use common::renderer::Renderer;
use common::scene::Scene;
use common::texture::Texture2D;

struct FrameState {
    pub last_frame_end: Instant,
    pub deltatime: f64,
    pub is_moving_camera: bool,
    pub fps: f32,
}

impl FrameState {
    pub fn update_statistics(&mut self) {
        self.deltatime = self.last_frame_end.elapsed().as_secs_f64();
        self.fps = (1.0 / self.deltatime) as f32;

        self.last_frame_end = Instant::now();
    }
}

impl Default for FrameState {
    fn default() -> Self {
        FrameState {
            last_frame_end: Instant::now(),
            deltatime: 0.0,
            fps: 0.0,
            is_moving_camera: false,
        }
    }
}

pub struct Game {
    input: Input,
    scene: Scene,
    renderer: Renderer,
    state: FrameState,
}

impl Application for Game {
    fn new(
        window: &Window,
        display: &Display<WindowSurface>,
        event_loop: &ActiveEventLoop,
    ) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        let inner_size = window.inner_size();
        let renderer =
            Renderer::new(inner_size.width as f32, inner_size.height as f32, display).unwrap();
        let mut scene =
            Scene::from_path(&PathBuf::from("assets/game_scenes/map.json"), display).unwrap();

        // scene.camera = scene.starting_camera.clone();

        // let inner_size = opengl_context.window.inner_size();
        /*scene.camera = Camera::new_fps(
            Point3::new(3.0, 0.2, 3.0),
            -Vector3::new(3.0, 0.2, 3.0).normalize(),
            inner_size.width as f32 / inner_size.height as f32,
        );*/

        scene.quads.add_node(Quad::new(
            Point2::new(0.1, 0.1),
            Vector2::new(0.2, 0.2),
            Texture2D::load(PathBuf::from("assets/textures/crosshair.png"), display).unwrap(),
            0,
        ));

        let state = FrameState::default();
        let input = Input::new();

        Self {
            renderer,
            scene,
            state,
            input,
        }
    }

    fn window_event(
        &mut self,
        event: WindowEvent,
        event_loop: &ActiveEventLoop,
        window: &Window,
        display: &Display<WindowSurface>,
    ) {
        self.input.process_window_event(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                display.resize((new_size.width, new_size.height));

                self.renderer
                    .update_projection_matrices(new_size.width as f32, new_size.height as f32);
            }
            WindowEvent::RedrawRequested => {
                if self.input.key_pressed(KeyCode::Escape) {
                    event_loop.exit();
                }

                self.update(window, display);
                self.render(window, display);

                self.state.update_statistics();
            }
            _ => (),
        };
    }

    fn device_event(
        &mut self,
        event: DeviceEvent,
        event_loop: &ActiveEventLoop,
        window: &Window,
        display: &Display<WindowSurface>,
    ) {
        self.input.process_device_event(event);
    }
}

impl Game {
    fn update(&mut self, window: &Window, display: &Display<WindowSurface>) {
        self.state.is_moving_camera = self.input.mouse_button_down(MouseButton::Middle)
            || self.input.key_down(KeyCode::Space);

        if self.state.is_moving_camera {
            self.scene
                .camera
                .update(&self.input, self.state.deltatime as f32);

            self.capture_cursor(window);
            window.set_cursor_visible(false);
            self.center_cursor(window);
        } else {
            self.release_cursor(window);
            window.set_cursor_visible(true);
        }

        self.input.reset_internal_state();
    }

    fn render(&mut self, window: &Window, display: &Display<WindowSurface>) {
        let mut target = display.draw();
        {
            self.scene.render(
                &mut self.renderer,
                &self.scene.camera.view(),
                self.scene.camera.position(),
                false,
                display,
                &mut target,
            );
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self, window: &Window, display: &Display<WindowSurface>) {}
}
