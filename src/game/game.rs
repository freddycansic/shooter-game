use crate::player::Player;
use cgmath::{InnerSpace, Point3, Vector3};
use common::app::Application;
use common::camera::Camera;
use common::context::OpenGLContext;
use common::debug;
use common::input::Input;
use common::renderer::Renderer;
use common::scene::Scene;
use std::fs;
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;

struct FrameState {
    pub last_frame_end: Instant,
    pub deltatime: f64,
    pub using_viewport: bool,
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
            using_viewport: false,
        }
    }
}

pub struct Game {
    input: Input,
    scene: Scene,
    player: Player,
    renderer: Renderer,
    opengl_context: OpenGLContext,
    state: FrameState,
}

impl Game {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        let opengl_context = OpenGLContext::new("We shootin now", false, event_loop);

        let renderer = Renderer::new(&opengl_context.display).unwrap();
        let mut scene = Scene::deserialize(
            &fs::read_to_string("assets/game_scenes/map.json").unwrap(),
            &opengl_context.display,
            opengl_context.window.inner_size(),
        )
        .unwrap();
        // scene.camera = scene.starting_camera.clone();

        let inner_size = opengl_context.window.inner_size();
        scene.camera = Camera::new_fps(
            Point3::new(3.0, 0.2, 3.0),
            -Vector3::new(3.0, 0.2, 3.0).normalize(),
            inner_size.width as f32 / inner_size.height as f32,
        );

        let state = FrameState::default();
        let input = Input::new();

        let player = Player::new();

        Self {
            opengl_context,
            renderer,
            scene,
            state,
            input,
            player,
        }
    }
}

impl Application for Game {
    // TODO figure out some way to not copy this code from editor
    fn run(mut self, event_loop: EventLoop<()>) {
        event_loop
            .run(move |event, event_loop_window_target| {
                event_loop_window_target.set_control_flow(ControlFlow::Poll);
                self.input
                    .process_event(self.opengl_context.window.id(), &event);

                match event {
                    Event::WindowEvent {
                        event: window_event,
                        window_id,
                    } if window_id == self.opengl_context.window.id() => {
                        match &window_event {
                            WindowEvent::CloseRequested => event_loop_window_target.exit(),
                            WindowEvent::Resized(new_size) => {
                                self.opengl_context
                                    .display
                                    .resize((new_size.width, new_size.height));
                                self.scene.camera.set_aspect_ratio(
                                    new_size.width as f32 / new_size.height as f32,
                                );
                            }
                            WindowEvent::RedrawRequested => {
                                if self.input.key_pressed(KeyCode::Escape) {
                                    event_loop_window_target.exit();
                                }

                                self.update();
                                self.render();

                                self.state.update_statistics();
                            }
                            _ => (),
                        };
                    }
                    Event::AboutToWait => self.opengl_context.window.request_redraw(),
                    _ => (),
                }
            })
            .unwrap();
    }

    fn update(&mut self) {
        self.state.using_viewport = true;

        if self.state.using_viewport {
            self.scene
                .camera
                .update(&self.input, self.state.deltatime as f32);
            self.player.update(self.state.deltatime as f32);

            self.opengl_context.capture_cursor();
            self.opengl_context.window.set_cursor_visible(false);
            self.opengl_context.center_cursor();
        } else {
            self.opengl_context.release_cursor();
            self.opengl_context.window.set_cursor_visible(true);
        }

        self.input.reset_internal_state();
    }

    fn render(&mut self) {
        let mut target = self.opengl_context.display.draw();
        {
            self.scene.render(
                &mut self.renderer,
                &self.opengl_context.display,
                &mut target,
            );
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self) {}
}
