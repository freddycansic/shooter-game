use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use egui_glium::EguiGlium;
use egui_glium::egui_winit::egui::{self, Align, Button, ViewportId};
use glium::Display;
use glium::glutin::surface::WindowSurface;
use log::info;
use palette::Srgb;
use rapier3d::na::{Point2, Point3, Vector2, Vector3};
use rfd::FileDialog;
use texture::Texture2D;
use winit::event::{DeviceEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use common::application::Application;
use common::camera::Camera;
use common::camera::OrbitalCamera;
use common::colors::{Color, ColorExt};
use common::light::Light;
use common::line::Line;
use common::renderer::Renderer;
use common::scene::Background;
use common::texture::Cubemap;
use common::*;
use input::Input;
use scene::Scene;

struct FrameState {
    pub last_frame_end: Instant,
    pub frame_count: u128,
    pub deltatime: f64,
    pub fps: f32,
    pub is_moving_camera: bool,
    pub gui: GuiState,
}

struct GuiState {
    pub render_lights: bool,
}

impl FrameState {
    pub fn update_statistics(&mut self) {
        self.frame_count = (self.frame_count + 1) % u128::MAX;

        self.deltatime = self.last_frame_end.elapsed().as_secs_f64();
        self.fps = (1.0 / self.deltatime) as f32;

        self.last_frame_end = Instant::now();
    }
}

enum EngineEvent {
    ImportHDRIBackground(PathBuf),
    LoadScene(String),
    ImportModel(PathBuf),
}

pub struct Editor {
    input: Input,
    scene: Scene,
    camera: OrbitalCamera,
    renderer: Renderer,
    gui: EguiGlium,
    state: FrameState,
    sender: Sender<EngineEvent>,
    receiver: Receiver<EngineEvent>,
}

impl Application for Editor {
    fn new(
        window: &Window,
        display: &Display<WindowSurface>,
        event_loop: &ActiveEventLoop,
    ) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        // TODO deferred rendering https://learnopengl.com/Advanced-Lighting/Deferred-Shading

        let mut scene = Scene {
            lines: vec![
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
            ],
            // terrain: Some(
            //     Terrain::load(
            //         &PathBuf::from("assets/terrain/terrain_heightmap.png"),
            //         &Material {
            //             diffuse: Texture2D::load(
            //                 PathBuf::from("assets/terrain/terrain_diffuse.jpg"),
            //                 display,
            //             )
            //             .unwrap(),
            //             ..Material::default(display).unwrap()
            //         },
            //         display,
            //     )
            //     .unwrap(),
            // ),
            ..Default::default()
        };

        // scene.quads.add_node(Quad::new(
        //     Point2::new(400.0, 300.0),
        //     Vector2::new(50.0, 50.0),
        //     Texture2D::default_diffuse(display).unwrap(),
        //     1,
        // ));
        // scene.quads.add_node(Quad::new(
        //     Point2::new(200.0, 200.0),
        //     Vector2::new(100.0, 100.0),
        //     Texture2D::load(PathBuf::from("assets/textures/crosshair.png"), display).unwrap(),
        //     1,
        // ));

        let camera = OrbitalCamera::default();

        // let mut model_instance = ModelInstance::from(
        //     Model::load(PathBuf::from("assets/models/cube.glb"), display).unwrap(),
        // );
        // model_instance.material = Some(Material {
        //     diffuse: Texture2D::load(PathBuf::from("assets/textures/container.png"), display)
        //         .unwrap(),
        //     specular: Texture2D::load(
        //         PathBuf::from("assets/textures/container_specular.png"),
        //         display,
        //     )
        //     .unwrap(),
        // });

        // scene.graph.add_node(model_instance.clone());
        // let child1 = scene.graph.add_node(model_instance.clone());
        // scene.graph.add_edge(root1, child1, ());
        //
        // let grandchild1 = scene.graph.add_node(model_instance.clone());
        // let grandchild2 = scene.graph.add_node(model_instance.clone());
        // scene.graph.add_edge(child1, grandchild1, ());
        // scene.graph.add_edge(child1, grandchild2, ());

        let inner_size = window.inner_size();
        let renderer =
            Renderer::new(inner_size.width as f32, inner_size.height as f32, display).unwrap();

        scene.lights.push(Light {
            position: Point3::new(3.0, 2.0, 1.0),
            color: Color::from_named(palette::named::WHITE),
        });

        // let size = 10;
        // let model_instance = ModelInstance::from(
        //     Model::load(PathBuf::from("assets/models/cube.glb"), display).unwrap(),
        // );

        // for x in -(size / 2)..(size / 2) {
        //     for y in -(size / 2)..(size / 2) {
        //         let mut m = model_instance.clone();
        //         m.transform.translation = Vector3::new(x as f32 * 6.0, y as f32 * 3.5, 0.0);

        //         scene.graph.add_node(m);
        //     }
        // }

        let input = Input::new();

        let gui = EguiGlium::new(ViewportId::ROOT, display, window, event_loop);

        let state = FrameState {
            last_frame_end: Instant::now(),
            frame_count: 0,
            deltatime: 0.0,
            fps: 0.0,
            is_moving_camera: false,
            gui: GuiState {
                render_lights: true,
            },
        };

        let (sender, receiver): (Sender<EngineEvent>, Receiver<EngineEvent>) = mpsc::channel();

        Self {
            scene,
            renderer,
            input,
            gui,
            state,
            sender,
            receiver,
            camera,
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

        let gui_event_response = self.gui.on_event(window, &event);

        if gui_event_response.repaint {
            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        device_event: DeviceEvent,
        _event_loop: &ActiveEventLoop,
        _window: &Window,
        _display: &Display<WindowSurface>,
    ) {
        self.input.process_device_event(device_event);
    }
}

impl Editor {
    fn update(&mut self, window: &Window, display: &Display<WindowSurface>) {
        for engine_event in self.receiver.try_iter() {
            match engine_event {
                EngineEvent::LoadScene(scene_string) => {
                    self.scene = Scene::from_string(&scene_string, display).unwrap()
                }
                EngineEvent::ImportModel(model_path) => self
                    .scene
                    .import_model(model_path.as_path(), display)
                    .unwrap(),
                EngineEvent::ImportHDRIBackground(hdri_directory_path) => {
                    self.scene.background =
                        Background::HDRI(Cubemap::load(hdri_directory_path, display).unwrap())
                }
            }
        }

        self.camera.update_zoom(&self.input);

        self.state.is_moving_camera = self.input.mouse_button_down(MouseButton::Middle)
            || self.input.key_down(KeyCode::Space);

        if self.state.is_moving_camera {
            self.camera.update(&self.input, self.state.deltatime as f32);
            self.capture_cursor(window);
            window.set_cursor_visible(false);
            self.center_cursor(window);
        } else {
            self.release_cursor(window);
            window.set_cursor_visible(true);
        }

        self.input.reset_internal_state();

        if self.state.frame_count % 5 == 0 {
            window.set_title(
                format!("Editing {} at {:.1} FPS", self.scene.title, self.state.fps).as_str(),
            );
        }
    }

    fn render(&mut self, window: &Window, display: &Display<WindowSurface>) {
        let window_size = window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        // let node_indices = self.scene.graph.node_indices().collect_vec();

        // self.scene.graph[node_indices[0]].transform.rotation =
        //     Quaternion::from_angle_y(Deg((self.state.frame_count % 360) as f32));

        let mut target = display.draw();
        {
            self.scene.render(
                &mut self.renderer,
                &self.camera.view(),
                self.camera.position(),
                self.state.gui.render_lights,
                display,
                &mut target,
            );

            self.render_gui(window);
            self.gui.paint(display, &mut target);
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self, window: &Window) {
        self.gui.run(window, |ctx| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.add(Button::new("New")).clicked() {
                                self.scene = Scene::default();

                                ui.close_menu();
                            }

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
                                info!("Saving scene...");
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
                                    .unwrap()
                                    .wait()
                                    .unwrap();

                                ui.close_menu();
                            }
                        });
                    });
                });
            });

            egui::SidePanel::left("left_panel")
                .default_width(100.0)
                .show(ctx, |ui| {
                    ui.collapsing("Models", |ui| {
                        if self.scene.graph.node_count() == 0 {
                            ui.label("There are no models in the scene.");
                        } else {
                            ui::collapsing_graph(ui, &mut self.scene.graph);
                        }
                    });

                    ui.add(egui::Separator::default().horizontal());

                    ui.collapsing("Quads", |ui| {
                        if self.scene.quads.node_count() == 0 {
                            ui.label("There are no quads in the scene.");
                        } else {
                            ui::collapsing_graph(ui, &mut self.scene.quads);
                        }
                    });
                });

            egui::SidePanel::right("right_panel").show(ctx, |ui| {
                ui.collapsing("Background", |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.scene.background,
                            Background::default(),
                            "Color",
                        );

                        if ui.selectable_label(false, "HDRI").clicked() {
                            let sender = self.sender.clone();

                            std::thread::spawn(move || {
                                if let Some(path) = FileDialog::new()
                                    .set_can_create_directories(true)
                                    .set_directory("/")
                                    .pick_folder()
                                {
                                    sender
                                        .send(EngineEvent::ImportHDRIBackground(path))
                                        .unwrap();
                                }
                            });
                        }
                    });
                });

                ui.collapsing("Lighting", |ui| {
                    ui.checkbox(&mut self.state.gui.render_lights, "Render lights");
                });
            });
        });
    }
}
