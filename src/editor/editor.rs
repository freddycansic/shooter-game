use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::Thread;
use std::time::Instant;

use cgmath::{Deg, Point3, Quaternion, Rotation3, Vector3};
use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::{Align, Button, ViewportId};
use egui_glium::egui_winit::winit::event_loop::EventLoop;
use egui_glium::EguiGlium;
use glium::glutin::surface::WindowSurface;
use glium::Display;
use image::open;
use palette::Srgb;
use rfd::FileDialog;
use serde::Serialize;
use winit::event::{Event, MouseButton, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::KeyCode;

use app::Application;
use common::camera::Camera;
use common::*;
use context::OpenGLContext;
use input::Input;
use line::Line;
use model::{Model, ModelInstance, Transform};
use scene::Scene;

struct FrameState {
    pub start: Instant,
    pub frame_count: u128,
    pub deltatime: f64,
    pub fps: f32,
    pub using_viewport: bool,
}

impl FrameState {
    pub fn update_statistics(&mut self) {
        self.frame_count = (self.frame_count + 1) % u128::MAX;

        self.deltatime = self.start.elapsed().as_secs_f64();
        self.fps = (1.0 / self.deltatime) as f32;
    }
}

enum EngineEvent {
    LoadScene(String),
    ImportModel(PathBuf),
}

pub struct Editor {
    input: Input,
    scene: Scene,
    opengl_context: OpenGLContext,
    gui: EguiGlium,
    state: FrameState,
    sender: Sender<EngineEvent>,
    receiver: Receiver<EngineEvent>,
}

impl Editor {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        // TODO deferred rendering https://learnopengl.com/Advanced-Lighting/Deferred-Shading
        let opengl_context = OpenGLContext::new("We glutin teapot now", false, event_loop);

        let mut scene = Scene::new("Untitled", Camera::default(), &opengl_context.display).unwrap();

        scene.lines = vec![
            Line::new(
                Point3::new(-1000.0, 0.0, 0.0),
                Point3::new(1000.0, 0.0, 0.0),
                Srgb::from(palette::named::RED),
                2,
            ),
            Line::new(
                Point3::new(0.0, -1000.0, 0.0),
                Point3::new(0.0, 1000.0, 0.0),
                Srgb::from(palette::named::GREEN),
                2,
            ),
            Line::new(
                Point3::new(0.0, 0.0, -1000.0),
                Point3::new(0.0, 0.0, 1000.0),
                Srgb::from(palette::named::BLUE),
                2,
            ),
        ];

        let input = Input::new();

        let gui = EguiGlium::new(
            ViewportId::ROOT,
            &opengl_context.display,
            &opengl_context.window,
            event_loop,
        );

        let state = FrameState {
            start: Instant::now(),
            frame_count: 0,
            deltatime: 0.0,
            fps: 0.0,
            using_viewport: false,
        };

        let (sender, receiver): (Sender<EngineEvent>, Receiver<EngineEvent>) = mpsc::channel();

        Self {
            opengl_context,
            scene,
            input,
            gui,
            state,
            sender,
            receiver,
        }
    }
}

impl Application for Editor {
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

                        let event_response = self
                            .gui
                            .on_event(&self.opengl_context.window, &window_event);

                        if event_response.repaint {
                            self.opengl_context.window.request_redraw();
                        }
                    }
                    Event::AboutToWait => self.opengl_context.window.request_redraw(),
                    _ => (),
                }
            })
            .unwrap();
    }

    fn update(&mut self) {
        for engine_event in self.receiver.try_iter() {
            match engine_event {
                EngineEvent::LoadScene(scene_string) => {
                    self.scene = Scene::deserialize(
                        &scene_string,
                        &self.opengl_context.display,
                        self.opengl_context.window.inner_size(),
                    )
                    .unwrap()
                }
                EngineEvent::ImportModel(model_path) => self
                    .scene
                    .import_model(model_path.as_path(), &self.opengl_context.display)
                    .unwrap(),
            }
        }

        self.state.using_viewport = self.input.mouse_button_down(MouseButton::Middle)
            || self.input.key_down(KeyCode::Space);

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

        if self.state.frame_count % 5 == 0 {
            self.opengl_context.window.set_title(
                format!("Editing {} at {:.1} FPS", self.scene.title, self.state.fps).as_str(),
            );
        }
    }

    fn render(&mut self) {
        let window_size = self.opengl_context.window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        for model_instance in self.scene.model_instances.iter_mut() {
            model_instance.transform.rotation =
                Quaternion::from_angle_y(Deg((self.state.frame_count % 360) as f32));
        }

        let mut target = self.opengl_context.display.draw();
        {
            self.scene.render(&self.opengl_context.display, &mut target);

            self.render_gui();

            self.gui.paint(&self.opengl_context.display, &mut target);
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self) {
        self.gui.run(&self.opengl_context.window, |ctx| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.add(Button::new("Open scene")).clicked() {
                                let sender = self.sender.clone();

                                std::thread::spawn(move || {
                                    if let Some(file) = FileDialog::new()
                                        .add_filter("json", &["json"])
                                        .set_can_create_directories(true)
                                        .set_directory("/")
                                        .pick_file()
                                    {
                                        let scene_string = std::fs::read_to_string(file).unwrap();

                                        sender.send(EngineEvent::LoadScene(scene_string)).unwrap();
                                    }
                                });

                                ui.close_menu();
                            }

                            if ui.add(Button::new("Save as")).clicked() {
                                self.scene.save_as();
                                ui.close_menu();
                            }
                        });

                        ui.menu_button("Scene", |ui| {
                            if ui.add(Button::new("Import models")).clicked() {
                                let sender = self.sender.clone();

                                std::thread::spawn(move || {
                                    if let Some(paths) = FileDialog::new()
                                        .add_filter("gltf", &["gltf", "glb"])
                                        .set_can_create_directories(true)
                                        .set_directory("/")
                                        .pick_files()
                                    {
                                        for path in paths {
                                            sender.send(EngineEvent::ImportModel(path)).unwrap();
                                        }
                                    }
                                });

                                ui.close_menu();
                            }
                        });

                        ui.menu_button("Run", |ui| {
                            if ui.add(Button::new("Run game")).clicked() {
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

            egui::SidePanel::left("my_side_panel").show(ctx, |ui| {});
        });
    }
}
