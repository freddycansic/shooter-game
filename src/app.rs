use std::f32::consts::PI;
use std::sync::Arc;
use crate::{camera, colors, context, debug, input, maths, model, scene};
use cgmath::{Deg, Matrix, Matrix4, Point3, Quaternion, Rad, Rotation3, SquareMatrix, Vector3, Vector4};
use color_eyre::Result;
use std::time::{Duration, Instant};
use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::ViewportId;
use egui_glium::egui_winit::winit::event_loop::EventLoop;
use egui_glium::EguiGlium;
use glium::{Frame, Surface, uniform};
use glium::uniforms::UniformBuffer;
use image::open;
use log::{debug, info};
use winit::event::{DeviceEvent, Event, MouseButton, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::KeyCode;

use input::Input;
use scene::Scene;
use context::{OpenGLContext, RenderingContext};
use crate::model::{Model, ModelInstance, Transform};

pub struct App {
    input: Input,
    scene: Scene,
    opengl_context: OpenGLContext,
    rendering_context: RenderingContext,
    gui: EguiGlium
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        // TODO deferred rendering https://learnopengl.com/Advanced-Lighting/Deferred-Shading
        let opengl_context = OpenGLContext::new("We glutin teapot now", false, event_loop);
        let rendering_context = RenderingContext::new(
            "assets/shaders/default.vert",
            "assets/shaders/default.frag",
            &opengl_context.display
        ).unwrap();

        let mut scene = Scene::new(camera::Camera::new_fps(
            Point3::new(5.0, 2.0, 5.0),
            Vector3::new(0.0, 0.0, 1.0),
        ));

        let teapot = Model::load("assets/models/teapot.glb", &opengl_context.display).unwrap();

        scene.model_instances.reserve(100);

        let square_size = 10;

        for x in 0..square_size {
            for y in 0..square_size {
                scene.model_instances.push(ModelInstance {
                    model: teapot.clone(),
                    transform: Transform {
                        translation: Vector3::new(x as f32, y as f32, 0.0),
                        ..Transform::default()
                    },
                })
            }
        }

        let input = Input::new();

        let gui = EguiGlium::new(ViewportId::ROOT, &opengl_context.display, &opengl_context.window, event_loop);

        Self {
            opengl_context,
            rendering_context,
            scene,
            input,
            gui
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let mut frame_state = FrameState {
            start: Instant::now(),
            recreate_swapchain: false,
            frame_count: 0,
            deltatime: 0.0,
            fps: 0.0,
            using_viewport: false,
        };

        event_loop
            .run(move |event, event_loop_window_target| {
                event_loop_window_target.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent {
                        event: window_event,
                        window_id,
                    } if window_id == self.opengl_context.window.id() => {
                        match &window_event {
                            WindowEvent::CloseRequested => event_loop_window_target.exit(),
                            WindowEvent::KeyboardInput { event, .. } => {
                                self.input.process_key_event(event.clone());
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                self.input.process_cursor_moved_window_event(position.clone());
                            }
                            WindowEvent::MouseInput { state, button, .. } => {
                                self.input.process_mouse_button_event(*button, *state);
                            }
                            WindowEvent::Resized(new_size) => {
                                self.opengl_context.display.resize((new_size.width, new_size.height));
                                self.scene.camera.set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
                            }
                            WindowEvent::RedrawRequested => {
                                frame_state.start = Instant::now();

                                if self.input.key_pressed(KeyCode::Escape) {
                                    event_loop_window_target.exit();
                                }

                                self.update(&mut frame_state);
                                self.render(&mut frame_state);

                                frame_state.update_statistics();
                            }
                            _ => (),
                        };

                        let event_response = self.gui.on_event(&self.opengl_context.window, &window_event);

                        if event_response.repaint {
                            self.opengl_context.window.request_redraw();
                        }
                    }
                    Event::AboutToWait => self.opengl_context.window.request_redraw(),
                    Event::DeviceEvent { event, .. } => {
                        if let DeviceEvent::MouseMotion { delta, .. } = event {
                            self.input.process_cursor_moved_device_event(delta.clone());
                        }
                    },
                    _ => ()
                }
            })
            .unwrap();
    }

    fn update(&mut self, frame_state: &mut FrameState) {
        frame_state.using_viewport = self.input.mouse_button_down(MouseButton::Middle);

        if frame_state.using_viewport {
            self.scene.camera.update(&self.input);
            self.opengl_context.capture_cursor();
            self.opengl_context.window.set_cursor_visible(false);
            self.opengl_context.center_cursor();
        } else {
            self.opengl_context.release_cursor();
            self.opengl_context.window.set_cursor_visible(true);
        }

        self.input.reset_internal_state();

        self.opengl_context.window.set_title(format!("{:.1} FPS", frame_state.fps).as_str());
    }

    fn render(&mut self, frame_state: &mut FrameState) {
        let window_size = self.opengl_context.window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        for model_instance in self.scene.model_instances.iter_mut() {
            model_instance.transform.rotation = Quaternion::from_angle_y(Deg((frame_state.frame_count % 360) as f32));
        }

        let mut target = self.opengl_context.display.draw();
        {
            self.scene.render(
                &self.rendering_context.program,
                &self.opengl_context.display,
                &mut target
            );

            self.render_gui(frame_state);

            self.gui.paint(&self.opengl_context.display, &mut target);
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self, frame_state: &mut FrameState) {
        self.gui.run(&self.opengl_context.window, |ctx| {
            egui::SidePanel::left("my_side_panel").show(ctx, |ui| {
                ui.heading("Hello World!");
                if ui.button("Quit").clicked() {

                }
            });
        });
    }
}

struct FrameState {
    start: Instant,
    recreate_swapchain: bool,
    frame_count: u128,
    deltatime: f64,
    fps: f32,
    using_viewport: bool
}

impl FrameState {
    fn update_statistics(&mut self) {
        self.frame_count = (self.frame_count + 1) % u128::MAX;

        self.deltatime = self.start.elapsed().as_secs_f64();
        self.fps = (1.0 / self.deltatime) as f32;
    }
}
