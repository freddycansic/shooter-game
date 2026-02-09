use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use common::debug::Cuboid;
use common::maths::{Ray, Transform};
use common::serde::SerializedWorld;
use common::world::{WorldGraph, WorldNode};
use egui_glium::EguiGlium;
use egui_glium::egui_winit::egui::{self, Align, Button, ViewportId};
use glium::Display;
use glium::glutin::surface::WindowSurface;
use itertools::Itertools;
use log::info;
use nalgebra::{Point3, Vector3, Vector4};
use palette::Srgb;
use petgraph::prelude::NodeIndex;
use rfd::FileDialog;
use uuid::Uuid;
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
use common::systems::renderer::{Background, Renderable, Renderer};
// use common::scene::Background;
use crate::ui::Show;
use common::collision::colliders::sphere::Sphere;
use common::components::component::Component;
use common::engine::Engine;
use common::resources::Resources;
use common::world::World;
use common::*;
use input::Input;

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
    pub debug_cube_index: usize,
    pub debug_cube_opacity: f32,
    pub render_debug_mouse_rays: bool,
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
    LoadProject(String),
    ImportModel(PathBuf),
}

pub struct Editor {
    engine: Engine,
    camera: OrbitalCamera,
    state: FrameState,
    sender: Sender<EngineEvent>,
    receiver: Receiver<EngineEvent>,
    debug_cuboids: Vec<Cuboid>,
    selection: Vec<NodeIndex>,
    world: World,
}

impl Application for Editor {
    fn new(window: &Window, display: &Display<WindowSurface>, event_loop: &ActiveEventLoop) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        // TODO deferred rendering https://learnopengl.com/Advanced-Lighting/Deferred-Shading

        let camera = OrbitalCamera::new(Point3::origin(), 5.0, 1920.0, 1080.0);

        let renderer = Renderer::new(
            None, // full size
            display,
        )
        .unwrap();

        let resources = Resources::new();

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
                debug_cube_index: 0,
                debug_cube_opacity: 0.5,
                render_debug_mouse_rays: false,
            },
        };

        let (sender, receiver): (Sender<EngineEvent>, Receiver<EngineEvent>) = mpsc::channel();

        let mut world = World::new();
        world.lights = vec![Light {
            position: Point3::new(3.0, 2.0, 1.0),
            color: Color::from_named(palette::named::WHITE),
        }];

        let engine = Engine {
            renderer,
            input,
            gui,
            resources,
        };

        Self {
            engine,
            state,
            sender,
            receiver,
            camera,
            world,
            debug_cuboids: vec![],
            selection: vec![],
        }
    }

    fn window_event(
        &mut self,
        event: WindowEvent,
        event_loop: &ActiveEventLoop,
        window: &Window,
        display: &Display<WindowSurface>,
    ) {
        self.engine.input.process_window_event(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                display.resize((new_size.width, new_size.height));

                self.camera
                    .update_projection_matrices(new_size.width as f32, new_size.height as f32);
            }
            WindowEvent::RedrawRequested => {
                if self.engine.input.key_pressed(KeyCode::Escape) {
                    event_loop.exit();
                }

                self.update(window, display);
                self.render(window, display);

                self.state.update_statistics();
            }
            _ => (),
        };

        let gui_event_response = self.engine.gui.on_event(window, &event);

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
        self.engine.input.process_device_event(device_event);
    }
}

impl Editor {
    fn update(&mut self, window: &Window, display: &Display<WindowSurface>) {
        let events = self.receiver.try_iter().collect_vec();

        for engine_event in events.into_iter() {
            match engine_event {
                EngineEvent::LoadProject(serialized_project) => {
                    // At the moment this just loads a world
                    // In the future it might be necessary to have multiple worlds in one project.
                    let serialized_world = serde_json::from_str::<SerializedWorld>(&serialized_project).unwrap();

                    self.world = serialized_world.into_world(display).unwrap();
                }
                EngineEvent::ImportModel(model_path) => self.import_model(model_path.as_path(), display).unwrap(),
                EngineEvent::ImportHDRIBackground(hdri_directory_path) => {
                    self.world.background = Background::HDRI(
                        self.engine
                            .resources
                            .get_cubemap_handle(&hdri_directory_path, display)
                            .unwrap(),
                    )
                }
            }
        }

        self.camera.update_zoom(&self.engine.input);

        self.state.is_moving_camera =
            self.engine.input.mouse_button_down(MouseButton::Middle) || self.engine.input.key_down(KeyCode::Space);

        if self.engine.input.mouse_button_just_released(MouseButton::Left)
            && self.engine.renderer.is_mouse_in_viewport(&self.engine.input)
        {
            log::warn!("Mouse click not implemented");
            // let ray = self.mouse_ray();
            //
            // let intersection = self.scene.intersect_ray(&ray);
            //
            // if self.state.gui.render_debug_mouse_rays {
            //     self.lines.push(Line::new(
            //         ray.origin,
            //         ray.origin + ray.direction() * 10.0,
            //         if intersection.is_some() {
            //             Srgb::new(0.0, 1.0, 0.0)
            //         } else {
            //             Srgb::new(1.0, 0.0, 0.0)
            //         },
            //         2,
            //     ));
            // }
            //
            // self.scene.graph.selection = match intersection {
            //     Some(node) => vec![node],
            //     None => vec![],
            // }
        }

        if self.state.is_moving_camera {
            self.camera.update(&self.engine.input, self.state.deltatime as f32);
            self.capture_cursor(window);
            window.set_cursor_visible(false);
            self.center_cursor(window);
        } else {
            self.release_cursor(window);
            window.set_cursor_visible(true);
        }

        self.engine.input.reset_internal_state();

        // if self.state.frame_count % 5 == 0 {
        //     info!("{} FPS", self.state.fps);
        //     window.set_title(
        //         format!("Editing {} at {:.1} FPS", self.scene.title, self.state.fps).as_str(),
        //     );
        // }
    }

    fn render(&mut self, window: &Window, display: &Display<WindowSurface>) {
        let window_size = window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        // for node in self.scene.graph.graph.node_weights_mut() {
        //     node.local_transform
        //         .set_rotation(UnitQuaternion::from_axis_angle(
        //             &Vector3::y_axis(),
        //             (self.state.frame_count as f32 * 0.001) % 360.0,
        //         ));
        // }

        let mut target = display.draw();
        {
            self.engine.renderer.render_world(
                &self.world,
                &self.camera,
                &self.engine.resources,
                &self.selection,
                display,
                &mut target,
            );

            self.render_gui(window);
            self.engine.gui.paint(display, &mut target);
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self, window: &Window) {
        self.engine.gui.run(window, |ctx| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.add(Button::new("New")).clicked() {
                                self.world = World::new();

                                ui.close();
                            }

                            if ui.add(Button::new("Open project")).clicked() {
                                let sender = self.sender.clone();

                                std::thread::spawn(move || {
                                    if let Some(file) = FileDialog::new()
                                        .add_filter("json", &["json"])
                                        .set_can_create_directories(true)
                                        .set_directory("/")
                                        .pick_file()
                                    {
                                        log::info!("Loading project {:?}", file);

                                        let project_string = std::fs::read_to_string(file).unwrap();

                                        sender.send(EngineEvent::LoadProject(project_string)).unwrap();
                                    }
                                });

                                ui.close();
                            }

                            if ui.add(Button::new("Save as")).clicked() {
                                info!("Saving project...");
                                // self.scene.save_as();
                                unimplemented!();
                                ui.close();
                            }
                        });

                        ui.menu_button("Project", |ui| {
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

                                ui.close();
                            }
                        });

                        ui.menu_button("Run", |ui| {
                            if ui.add(Button::new("Run game")).clicked() {
                                let uuid = Uuid::new_v4().to_string();
                                let mut temp_path = std::env::temp_dir();
                                temp_path.push(uuid.clone());

                                unimplemented!();
                                // let serialized_scene = SerializedScene::from_scene(&self.scene);
                                // let serialized_string = serde_json::to_string(&serialized_scene).unwrap();
                                //
                                // std::fs::write(&temp_path, serialized_string).unwrap();
                                //
                                // std::process::Command::new("cargo")
                                //     .arg("run")
                                //     .arg("--package")
                                //     .arg("game")
                                //     .arg("--")
                                //     .arg("--scene")
                                //     .arg(uuid)
                                //     .spawn()
                                //     .unwrap()
                                //     .wait()
                                //     .unwrap();
                                //
                                // ui.close();
                            }
                        });
                    });
                });
            });

            egui::SidePanel::left("left_panel")
                .default_width(100.0)
                .show(ctx, |ui| {
                    self.world.graph.show(ui);

                    ui.add(egui::Separator::default().horizontal());

                    // ui.collapsing("Quads", |ui| {
                    //     if self.scene.quads.node_count() == 0 {
                    //         ui.label("There are no quads in the scene.");
                    //     } else {
                    //         ui::collapsing_graph(ui, &mut self.scene.quads);
                    //     }
                    // });
                });

            egui::SidePanel::right("right_panel").show(ctx, |ui| {
                // ui.collapsing("Properties", |ui| {
                //     if self.scene.graph.selection.len() == 1 {
                //         let selected_node_index = self.scene.graph.selection[0];
                //         let selected_node = &mut self.scene.graph.graph[selected_node_index];
                //
                //         selected_node.local_transform.show(ui);
                //
                //         ui.separator();
                //
                //         ui.label("Components");
                //
                //         // TODO
                //         for component in &selected_node.components {
                //             ui.label(component.name());
                //         }
                //
                //         if ui.button("+").clicked() {
                //             selected_node.components.push(Component::PlayerSpawn);
                //         }
                //     }
                // });

                ui.collapsing("Debug", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.state.gui.debug_cube_index, 0..=self.debug_cuboids.len() - 1)
                            .integer(),
                    );

                    ui.add(egui::Slider::new(&mut self.state.gui.debug_cube_opacity, 0.0..=1.0));

                    ui.checkbox(&mut self.state.gui.render_debug_mouse_rays, "Render debug mouse rays");
                    if ui.button("Clear lines").clicked() {
                        // self.engine.renderer.lines.clear();
                        unimplemented!();
                    }
                });

                ui.separator();

                ui.collapsing("Background", |ui| {
                    ui.horizontal(|ui| {
                        // ui.selectable_value(
                        //     &mut self.scene.background,
                        //     Background::default(),
                        //     "Color",
                        // );

                        if ui.selectable_label(false, "HDRI").clicked() {
                            let sender = self.sender.clone();

                            std::thread::spawn(move || {
                                if let Some(path) = FileDialog::new()
                                    .set_can_create_directories(true)
                                    .set_directory("/")
                                    .pick_folder()
                                {
                                    sender.send(EngineEvent::ImportHDRIBackground(path)).unwrap();
                                }
                            });
                        }
                    });
                });

                ui.collapsing("Lighting", |ui| {
                    ui.checkbox(&mut self.state.gui.render_lights, "Render lights");
                });
            });

            // Update the viewport size with the amount of space after then panels have been added
            self.engine
                .renderer
                .update_viewport(ctx.available_rect(), &mut self.camera);
        });
    }

    fn save_as(&self) {
        unimplemented!()
        // let serialized_scene = SerializedScene::from_scene(self);
        //
        // let serialized = serde_json::to_string(&serialized_scene).unwrap();
        //
        // std::thread::spawn(move || {
        //     if let Some(save_path) = FileDialog::new().save_file() {
        //         std::fs::write(save_path, serialized).unwrap();
        //     }
        // });
    }

    /// Load a models and create an instance of it in the world
    fn import_model(&mut self, path: &Path, display: &Display<WindowSurface>) -> color_eyre::Result<()> {
        let handles = self.engine.resources.get_geometry_handles(path, display)?;

        let group_node = self.world.graph.add_root_node(WorldNode::default());

        let texture_handle = self
            .engine
            .resources
            .get_texture_handle(Path::new("assets/textures/uv-test.jpg"), display)?;

        for geometry_handle in handles {
            let world_node = WorldNode::default();
            let world_graph_node = self.world.graph.add_node(world_node);
            self.world.graph.add_edge(group_node, world_graph_node);

            let renderable = Renderable {
                geometry_handle,
                texture_handle,
                node: world_graph_node,
            };

            self.world.renderables.push(renderable);
        }

        Ok(())
    }

    fn mouse_ray(&self) -> Ray {
        // mouse coordinates in window coordinates
        let mouse = self.engine.input.mouse_position().unwrap();

        // mouse coordinates in viewport coordinates
        let viewport = self.engine.renderer.viewport.unwrap();
        let x_in_viewport = (mouse.x as f32) - viewport.left();
        let y_in_viewport = (mouse.y as f32) - viewport.top();

        // mouse coordinates in ndc coordinates (-1..1)
        let x_ndc = maths::linear_map(x_in_viewport, 0.0, viewport.width(), -1.0, 1.0);

        // for y, 1 is top and -1 is bottom
        let y_ndc = maths::linear_map(y_in_viewport, 0.0, viewport.height(), 1.0, -1.0);

        let vp = self.camera.perspective_projection() * self.camera.view();
        let inv_vp = vp.try_inverse().unwrap();

        // position of mouse coordinate on near and far plane in clip space
        let near_clip = Vector4::new(x_ndc, y_ndc, -1.0, 1.0);
        let far_clip = Vector4::new(x_ndc, y_ndc, 1.0, 1.0);

        // unproject to get points in world space
        let near_world_h = inv_vp * near_clip;
        let far_world_h = inv_vp * far_clip;

        // convert homogenous coordinates into cartesian
        let near_world = near_world_h.xyz() / near_world_h.w;
        let far_world = far_world_h.xyz() / far_world_h.w;

        let origin = near_world;
        let direction = (far_world - near_world).normalize();

        Ray::new(origin.into(), direction.into())
    }
}
