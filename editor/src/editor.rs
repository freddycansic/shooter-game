use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use common::debug::Cuboid;
use common::maths::{Ray, Transform};
use common::serde::SerializedWorld;
use common::world::WorldNode;
use egui_glium::egui_winit::egui::{self, Align, Button, Pos2};
use glium::Display;
use glium::glutin::surface::WindowSurface;
use itertools::Itertools;
use log::info;
use nalgebra::{Matrix4, Point3, Vector2, Vector4};
use palette::Srgb;
use petgraph::prelude::NodeIndex;
use rfd::FileDialog;
use uuid::Uuid;
use winit::event::{DeviceEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::ui::Show;
use common::application::Application;
use common::camera::OrbitalCamera;
use common::colors::{Color, ColorExt};
use common::context::WindowSize;
use common::ecs::entity::Entity;
use common::ecs::subsystem::Subsystem;
use common::ecs::system_parameters::event::{EventReader, EventWriter};
use common::ecs::system_parameters::query::Query;
use common::ecs::system_parameters::res::{Res, ResMut};
use common::engine::assets::Assets;
use common::engine::engine::Engine;
use common::engine::input::Input;
use common::engine::renderer::{Background, Viewport, ViewportChanged};
use common::executor::{CommandExecutor, RuntimeExecutor};
use common::gui::GuiState;
use common::light::Light;
use common::line::Line;
use common::subsystems::frame_timing::{FrameTiming, WinitNewEvents};
use common::window::{WindowResized, WinitWindowEvent};
use common::world::World;
use common::world::physics_context::ColliderSet;
use common::*;
use common_macros::{Event, Resource};

enum EngineEvent {
    ImportHDRIBackground(PathBuf),
    LoadProject(String),
    ImportModel(PathBuf),
}

#[derive(Event)]
struct ViewportClick {
    mouse_ray: Ray,
}

#[derive(Resource)]
struct Selection(Vec<Entity>);

pub struct Editor {
    engine: Engine,
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

        let mut world = World::default();
        world.register_resource(WindowSize {
            width: window.inner_size().width,
            height: window.inner_size().height,
        });

        let engine = Engine::new(None /* full size*/, display, window, event_loop);

        world.lights = vec![Light {
            position: Point3::new(3.0, 2.0, 1.0),
            color: Color::from_named(palette::named::WHITE),
        }];

        let (sender, receiver): (Sender<EngineEvent>, Receiver<EngineEvent>) = mpsc::channel();

        let mut editor = Self {
            engine,
            sender,
            receiver,
            world,
            debug_cuboids: vec![],
            selection: vec![],
        };

        let mut assets = Assets::new();
        assets.initialise_default_texture(&display).unwrap();
        editor.world.register_resource(assets);

        editor.register_subsystem::<FrameTiming>();
        editor.register_subsystem::<Input>();
        editor.engine.scheduler.register_continuous(Self::detect_viewport_click);
        editor
            .engine
            .scheduler
            .register_triggered::<ViewportClick, _, _>(Self::selection_stuff);

        editor
    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }

    fn run_systems(&mut self, mut executor: RuntimeExecutor, display: &Display<WindowSurface>) {
        let input = self.world.resource::<Input>().unwrap();

        if input.key_pressed(KeyCode::Escape) {
            executor.exit();
        }

        // TODO turn this into ECS systems
        self.update(display);

        self.engine.scheduler.run_systems(&mut self.world, &mut executor);
    }

    fn render(&mut self, event_loop: &ActiveEventLoop, window: &Window, display: &Display<WindowSurface>) {
        self.render(window, display);
    }

    fn window_event(
        &mut self,
        event: WindowEvent,
        _event_loop: &ActiveEventLoop,
        window: &Window,
        _display: &Display<WindowSurface>,
    ) {
        let gui_event_response = self.engine.gui.on_event(window, &event);

        if gui_event_response.repaint {
            window.request_redraw();
        }
    }
}

impl Editor {
    // TODO figure out where to put this
    pub fn register_subsystem<S>(&mut self)
    where
        S: Subsystem,
    {
        S::register_resources(&mut self.world);
        S::register_systems(&mut self.engine.scheduler);
    }

    // TODO turn all this into ECS event stuff
    fn update(&mut self, display: &Display<WindowSurface>) {
        let events = self.receiver.try_iter().collect_vec();

        // TODO turn these into executor events
        for engine_event in events.into_iter() {
            match engine_event {
                EngineEvent::LoadProject(serialized_project) => {
                    // At the moment this just loads a world
                    // In the future it might be necessary to have multiple worlds in one project.
                    let serialized_world = serde_json::from_str::<SerializedWorld>(&serialized_project).unwrap();

                    self.world = serialized_world.into_world(display, &mut self.engine.assets).unwrap();
                }
                EngineEvent::ImportModel(model_path) => self.import_model(model_path.as_path(), display).unwrap(),
                EngineEvent::ImportHDRIBackground(hdri_directory_path) => {
                    self.world.background = Background::HDRI(
                        self.engine
                            .assets
                            .get_cubemap_handle(&hdri_directory_path, display)
                            .unwrap(),
                    )
                }
            }
        }

        input.reset_internal_state();

        // if self.state.frame_count % 5 == 0 {
        //     info!("{} FPS", self.state.fps);
        //     window.set_title(
        //         format!("Editing {} at {:.1} FPS", self.scene.title, self.state.fps).as_str(),
        //     );
        // }
    }

    fn detect_viewport_click(
        input: Res<Input>,
        viewport: Res<Viewport>,
        camera: Res<OrbitalCamera>,
        mut viewport_click: EventWriter<ViewportClick>,
    ) {
        let mouse_in_viewport = Self::is_mouse_in_viewport(&input, viewport.0);
        let left_just_released = input.mouse_button_just_released(MouseButton::Left);

        if left_just_released && mouse_in_viewport {
            let mouse_ray = mouse_ray(input.mouse_position(), &camera.inv_vp(), viewport.0.unwrap());

            viewport_click.write(ViewportClick { mouse_ray });
        }
    }

    pub fn is_mouse_in_viewport(input: &Input, viewport: &Viewport) -> bool {
        if !input.mouse_on_window() {
            return false;
        }

        let mouse_position = input.mouse_position();

        viewport.is_some_and(|viewport| viewport.contains(Pos2::new(mouse_position.x as f32, mouse_position.y as f32)))
    }

    fn selection_stuff(
        gui_state: Res<GuiState>,
        mut viewport_click: EventReader<Viewport>,
        assets: Res<Assets>,
        mut selection: ResMut<Selection>,
        mut colliders: Query<(&ColliderSet, &Transform)>,
    ) {
        let mouse_ray = viewport_click.read().next().unwrap();

        for (collider_set, transform) in colliders.iter() {
            
        }

        let intersection = self.world.raycast(&mouse_ray, &assets);

        if gui_state.render_debug_mouse_rays {
            self.world.lines.push(Line::new(
                ray.origin,
                ray.origin + ray.direction() * 1000.0,
                if intersection.is_some() {
                    Srgb::new(0.0, 1.0, 0.0)
                } else {
                    Srgb::new(1.0, 0.0, 0.0)
                },
                2,
            ));
        }

        selection = match intersection {
            Some(hit) => vec![hit.node],
            None => vec![],
        };
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
            self.world.graph.calculate_world_matrices();
            self.engine.renderer.render_world(
                &self.world,
                &self.camera,
                &self.engine.assets,
                &self.selection,
                display,
                &mut target,
            );

            self.render_gui(window);
            self.engine.gui.paint(display, &mut target);
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self, window: &Window, mut viewport_changed: EventWriter<ViewportChanged>) {
        self.engine.gui.run(window, |ctx| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.add(Button::new("New")).clicked() {
                                self.world = World::default();

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
                                self.world.save_as(&self.engine.assets);
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

                                let serialized_world = SerializedWorld::from_world(&self.world, &self.engine.assets);
                                let serialized_string = serde_json::to_string(&serialized_world).unwrap();

                                std::fs::write(&temp_path, serialized_string).unwrap();

                                std::process::Command::new("cargo")
                                    .arg("run")
                                    .arg("--package")
                                    .arg("game")
                                    .arg("--")
                                    .arg("--project")
                                    .arg(uuid)
                                    .spawn()
                                    .unwrap()
                                    .wait()
                                    .unwrap();

                                ui.close();
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
                ui.collapsing("Properties", |ui| {
                    if self.selection.len() == 1 {
                        let selected_node_index = self.selection[0];
                        let selected_node = &mut self.world.graph.graph[selected_node_index];

                        selected_node.local_transform.show(ui);

                        dbg!(&selected_node.local_transform);

                        ui.label(format!("Node index: {:?}", selected_node_index));

                        ui.separator();

                        ui.label("Components");

                        if self.world.player_spawn == Some(selected_node_index) {
                            ui.label("Player spawn");
                        }
                        if self.world.physics_context.colliders.contains_key(&selected_node_index) {
                            ui.horizontal(|ui| {
                                ui.label("Collider");
                                if ui.button("-").clicked() {
                                    self.world.physics_context.colliders.remove(&selected_node_index);
                                }
                            });
                        }

                        if ui.button("+").clicked() {
                            self.world.player_spawn = Some(selected_node_index);
                        }
                    }
                });

                ui.collapsing("Debug", |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.state.gui.debug_cube_index, 0..=self.debug_cuboids.len()).integer(),
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
            viewport_changed.write(ViewportChanged(Viewport(Some(ctx.available_rect()))));
        });
    }

    /// Load a models and create an instance of it in the world
    fn import_model(&mut self, path: &Path, display: &Display<WindowSurface>) -> color_eyre::Result<()> {
        let handles = self.engine.assets.get_geometry_handles(path, Some(display))?;

        let group_node = self.world.graph.add_root_node(WorldNode::default());

        for geometry_handle in handles {
            let world_node = WorldNode::default();
            let world_graph_node = self.world.graph.add_node(world_node);
            self.world.graph.add_edge(group_node, world_graph_node);

            self.world
                .physics_context
                .colliders
                .insert(world_graph_node, ColliderSet::from(geometry_handle));
            self.world.geometries.insert(world_graph_node, geometry_handle);
        }

        Ok(())
    }
}

fn mouse_ray(mouse_position: Vector2<f64>, inv_vp: &Matrix4<f32>, viewport: egui::Rect) -> Ray {
    // mouse coordinates in window coordinates
    // mouse_position

    // mouse coordinates in viewport coordinates
    let x_in_viewport = (mouse_position.x as f32) - viewport.left();
    let y_in_viewport = (mouse_position.y as f32) - viewport.top();

    // mouse coordinates in ndc coordinates (-1..1)
    let x_ndc = maths::linear_map(x_in_viewport, 0.0, viewport.width(), -1.0, 1.0);

    // for y, 1 is top and -1 is bottom
    let y_ndc = maths::linear_map(y_in_viewport, 0.0, viewport.height(), 1.0, -1.0);

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
