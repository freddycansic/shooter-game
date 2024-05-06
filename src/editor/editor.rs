use cgmath::{Deg, Matrix, Matrix4, Point3, Quaternion, Rad, Rotation3, SquareMatrix, Vector3, Vector4};
use std::time::{Duration, Instant};
use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::{Align, ViewportId};
use egui_glium::egui_winit::winit::event_loop::EventLoop;
use egui_glium::EguiGlium;
use glium::{Frame, Surface, uniform};
use winit::event::{DeviceEvent, Event, MouseButton, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::KeyCode;

use common::*;
use input::Input;
use scene::Scene;
use context::{OpenGLContext, RenderingContext};
use model::{Model, ModelInstance, Transform};
use app::Application;
use debug;

struct FrameState {
    pub start: Instant,
    pub frame_count: u128,
    pub deltatime: f64,
    pub fps: f32,
    pub using_viewport: bool
}

impl FrameState {
    pub fn update_statistics(&mut self) {
        self.frame_count = (self.frame_count + 1) % u128::MAX;

        self.deltatime = self.start.elapsed().as_secs_f64();
        self.fps = (1.0 / self.deltatime) as f32;
    }
}

pub struct Editor {
    input: Input,
    scene: Scene,
    opengl_context: OpenGLContext,
    rendering_context: RenderingContext,
    gui: EguiGlium,
    state: FrameState,
}

impl Editor {
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

        let square_size = 10_u32;
        scene.model_instances.reserve(square_size.pow(2) as usize);

        for x in 0..square_size {
            for y in 0..square_size {
                scene.model_instances.push(ModelInstance {
                    model: teapot.clone(),
                    transform: Transform {
                        translation: Vector3::new(x as f32 * 6.0, y as f32 * 4.0, 0.0),
                        ..Transform::default()
                    },
                })
            }
        }

        let input = Input::new();

        let gui = EguiGlium::new(ViewportId::ROOT, &opengl_context.display, &opengl_context.window, event_loop);

        let state = FrameState {
            start: Instant::now(),
            frame_count: 0,
            deltatime: 0.0,
            fps: 0.0,
            using_viewport: false,
        };

        Self {
            opengl_context,
            rendering_context,
            scene,
            input,
            gui,
            state,
        }
    }
}

impl Application for Editor {
    fn run(mut self, event_loop: EventLoop<()>) {
        event_loop
            .run(move |event, event_loop_window_target| {
                event_loop_window_target.set_control_flow(ControlFlow::Poll);
                self.input.process_event(self.opengl_context.window.id(), &event);

                match event {
                    Event::WindowEvent {
                        event: window_event,
                        window_id,
                    } if window_id == self.opengl_context.window.id() => {
                        match &window_event {
                            WindowEvent::CloseRequested => event_loop_window_target.exit(),
                            WindowEvent::Resized(new_size) => {
                                self.opengl_context.display.resize((new_size.width, new_size.height));
                                self.scene.camera.set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
                            }
                            WindowEvent::RedrawRequested => {
                                self.state.start = Instant::now();

                                if self.input.key_pressed(KeyCode::Escape) {
                                    event_loop_window_target.exit();
                                }

                                self.update();
                                self.render();

                                self.state.update_statistics();
                            }
                            _ => (),
                        };

                        let event_response = self.gui.on_event(&self.opengl_context.window, &window_event);

                        if event_response.repaint {
                            self.opengl_context.window.request_redraw();
                        }
                    }
                    Event::AboutToWait => self.opengl_context.window.request_redraw(),
                    _ => ()
                }
            })
            .unwrap();
    }

    fn update(&mut self) {
        self.state.using_viewport =
            self.input.mouse_button_down(MouseButton::Middle) ||
            self.input.key_down(KeyCode::KeyD);

        if self.state.using_viewport {
            self.scene.camera.update(&self.input);
            self.opengl_context.capture_cursor();
            self.opengl_context.window.set_cursor_visible(false);
            self.opengl_context.center_cursor();
        } else {
            self.opengl_context.release_cursor();
            self.opengl_context.window.set_cursor_visible(true);
        }

        self.input.reset_internal_state();

        self.opengl_context.window.set_title(format!("{:.1} FPS", self.state.fps).as_str());
    }

    fn render(&mut self) {
        let window_size = self.opengl_context.window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        for model_instance in self.scene.model_instances.iter_mut() {
            model_instance.transform.rotation = Quaternion::from_angle_y(Deg((self.state.frame_count % 360) as f32));
        }

        let mut target = self.opengl_context.display.draw();
        {
            self.scene.render(
                &self.rendering_context.program,
                &self.opengl_context.display,
                &mut target
            );

            self.render_gui();

            self.gui.paint(&self.opengl_context.display, &mut target);
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self) {
        self.gui.run(&self.opengl_context.window, |ctx| {
            egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.add(egui::Button::new("Import model")).clicked() {
                                // Scene::import_model()
                                ui.close_menu();
                            }
                        });

                        ui.menu_button("Run", |ui| {
                            if ui.add(egui::Button::new("Run game")).clicked() {
                                std::process::Command::new("cargo")
                                    .arg("run")
                                    .arg("--package")
                                    .arg("shooter-game")
                                    .arg("--bin")
                                    .arg("game")
                                    .spawn()
                                    .unwrap();

                                ui.close_menu();
                            }
                        });
                    });
                });
            });

            egui::SidePanel::left("my_side_panel").show(ctx, |ui| {
            });
        });
    }
}
