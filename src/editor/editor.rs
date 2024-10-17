use cgmath::Point3;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::{Align, Button, CollapsingHeader, Ui, ViewportId};
use egui_glium::egui_winit::winit::event_loop::EventLoop;
use egui_glium::EguiGlium;
use itertools::Itertools;
use log::info;
use palette::Srgb;
use petgraph::graph::node_index;
use petgraph::prelude::StableDiGraph;
use petgraph::stable_graph::{EdgeReference, EdgeReferences, NodeIndex};
use petgraph::visit::{Bfs, EdgeRef, IntoEdgeReferences, IntoNodeReferences};
use petgraph::Direction;
use rfd::FileDialog;
use winit::event::{Event, MouseButton, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::KeyCode;

use app::Application;
use common::line::Line;
use common::model::Model;
use common::model_instance::ModelInstance;
use common::renderer::Renderer;
use common::*;
use context::OpenGLContext;
use input::Input;
use scene::Scene;

struct FrameState {
    pub last_frame_end: Instant,
    pub frame_count: u128,
    pub deltatime: f64,
    pub fps: f32,
    pub using_viewport: bool,
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
    LoadScene(String),
    ImportModel(PathBuf),
}

pub struct Editor {
    input: Input,
    scene: Scene,
    renderer: Renderer,
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
        let opengl_context = OpenGLContext::new("We glium teapot now", false, event_loop);

        let mut scene = Scene::default();
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

        let model_instance = ModelInstance::from(
            Model::load(
                PathBuf::from("assets/models/cube.glb"),
                &opengl_context.display,
            )
            .unwrap(),
        );

        let root1 = scene.graph.add_node(model_instance.clone());
        let child1 = scene.graph.add_node(model_instance.clone());
        scene.graph.add_edge(root1, child1, ());

        let grandchild1 = scene.graph.add_node(model_instance.clone());
        let grandchild2 = scene.graph.add_node(model_instance.clone());
        scene.graph.add_edge(child1, grandchild1, ());
        scene.graph.add_edge(child1, grandchild2, ());

        let renderer = Renderer::new(&opengl_context.display).unwrap();

        // let size = 10;
        // let model =
        //     model::load("assets/models/teapot.glb".into(), &opengl_context.display).unwrap();
        //
        // for x in -(size / 2)..(size / 2) {
        //     for y in -(size / 2)..(size / 2) {
        //         scene.model_instances.push(ModelInstance {
        //             model: model.clone(),
        //             texture: None,
        //             transform: Transform {
        //                 translation: Vector3::new(x as f32 * 6.0, y as f32 * 3.5, 0.0),
        //                 ..Transform::default()
        //             },
        //         });
        //     }
        // }

        let input = Input::new();

        let gui = EguiGlium::new(
            ViewportId::ROOT,
            &opengl_context.display,
            &opengl_context.window,
            event_loop,
        );

        let state = FrameState {
            last_frame_end: Instant::now(),
            frame_count: 0,
            deltatime: 0.0,
            fps: 0.0,
            using_viewport: false,
        };

        let (sender, receiver): (Sender<EngineEvent>, Receiver<EngineEvent>) = mpsc::channel();

        Self {
            opengl_context,
            scene,
            renderer,
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
                    self.scene =
                        Scene::from_string(&scene_string, &self.opengl_context.display).unwrap()
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
            self.scene
                .camera
                .update(&self.input, self.state.deltatime as f32);
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

        // for model_instance in self.scene.model_instances.iter_mut() {
        //     model_instance.transform.rotation =
        //         Quaternion::from_angle_y(Deg((self.state.frame_count % 360) as f32));
        // }

        let mut target = self.opengl_context.display.draw();
        {
            self.scene.render(
                &mut self.renderer,
                &self.opengl_context.display,
                &mut target,
            );

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
                                    .unwrap();

                                ui.close_menu();
                            }
                        });
                    });
                });
            });

            egui::SidePanel::left("my_side_panel").show(ctx, |ui| {
                let top_level_nodes = self
                    .scene
                    .graph
                    .node_references()
                    .filter(|(node_index, _)| {
                        self.scene
                            .graph
                            .neighbors_directed(*node_index, Direction::Incoming)
                            .count()
                            == 0
                    })
                    .map(|(node_index, _)| node_index)
                    .collect_vec();

                for (i, node) in top_level_nodes.iter().enumerate() {
                    let mut bfs = Bfs::new(&self.scene.graph, *node);

                    ui.push_id(i, |ui| {
                        if let Some(next) = bfs.next(&self.scene.graph) {
                            make_collapsing_header(ui, &mut self.scene.graph, next);
                        }
                    });
                }
            });
        });
    }
}

fn make_collapsing_header(
    ui: &mut Ui,
    graph: &mut StableDiGraph<ModelInstance, ()>,
    node_index: NodeIndex,
) {
    let model_name = graph[node_index].name.clone();
    let children = graph
        .neighbors_directed(node_index, Direction::Outgoing)
        .collect_vec();
    let id = ui.make_persistent_id(node_index);

    if children.is_empty() {
        ui.indent(id, |ui| {
            if ui.selectable_label(false, model_name).clicked() {
                graph[node_index].selected = !graph[node_index].selected;
            }
        });
    } else {
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                if ui.selectable_label(false, model_name).clicked() {
                    graph[node_index].selected = !graph[node_index].selected;
                }
            })
            .body(|ui| {
                for child in children.into_iter() {
                    make_collapsing_header(ui, graph, child);
                }
            });
    }
}
