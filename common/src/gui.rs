use crate::ecs::subsystem::Subsystem;
use crate::ecs::system_parameters::application_context::ApplicationContext;
use crate::ecs::system_parameters::event::{EventReader, EventWriter};
use crate::ecs::system_parameters::res::ResMut;
use crate::engine::scheduler::Stage;
use common::engine::renderer::{Viewport, ViewportChanged};
use common::engine::scheduler::Scheduler;
use common::world::World;
use common_macros::Resource;
use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::{Align, Button, ViewportId};
use egui_glium::EguiGlium;
use log::info;
use crate::runtime::{RuntimeContext, WinitWindowEvent};

// TODO move gui state into its own resource
#[derive(Resource)]
pub struct GuiState {
    pub render_lights: bool,
    pub debug_cube_index: usize,
    pub debug_cube_opacity: f32,
    pub render_debug_mouse_rays: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        GuiState {
            render_lights: true,
            debug_cube_index: 0,
            debug_cube_opacity: 0.5,
            render_debug_mouse_rays: false,
        }
    }
}

#[derive(Resource)]
pub struct Gui(pub EguiGlium);

impl Subsystem for Gui {
    fn register_resources(world: &mut World, context: Option<&RuntimeContext>) {
        world.register_resource(Gui(EguiGlium::new(
            ViewportId::ROOT,
            context.unwrap().display,
            context.unwrap().window,
            context.unwrap().event_loop,
        )));

        world.register_resource(GuiState::default());
    }

    fn register_systems(scheduler: &mut Scheduler) {
        scheduler.register_triggered::<WinitWindowEvent, _, _>(Gui::winit_window_event, Stage::Pre);
        scheduler.register_continuous(Gui::show, Stage::Main);
    }
}

impl Gui {
    fn winit_window_event(
        mut gui: ResMut<Gui>,
        commands: ApplicationContext,
        mut winit_window_event: EventReader<WinitWindowEvent>,
    ) {
        for event in winit_window_event.read() {
            let gui_event_response = gui.0.on_event(commands.window(), &event.0);

            if gui_event_response.repaint {
                commands.window().request_redraw();
            }
        }
    }

    fn show(mut gui: ResMut<Gui>, commands: ApplicationContext, mut viewport_changed: EventWriter<ViewportChanged>) {
        gui.0.run(commands.window(), |ctx| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.add(Button::new("New")).clicked() {
                                // self.world = World::default();
                                unimplemented!();
                                ui.close();
                            }

                            if ui.add(Button::new("Open project")).clicked() {
                                unimplemented!();

                                // let sender = self.sender.clone();
                                //
                                // std::thread::spawn(move || {
                                //     if let Some(file) = FileDialog::new()
                                //         .add_filter("json", &["json"])
                                //         .set_can_create_directories(true)
                                //         .set_directory("/")
                                //         .pick_file()
                                //     {
                                //         log::info!("Loading project {:?}", file);
                                //
                                //         let project_string = std::fs::read_to_string(file).unwrap();
                                //
                                //         sender.send(EngineEvent::LoadProject(project_string)).unwrap();
                                //     }
                                // });

                                ui.close();
                            }

                            if ui.add(Button::new("Save as")).clicked() {
                                info!("Saving project...");
                                unimplemented!();

                                // self.world.save_as(&self.engine.assets);
                            }
                        });

                        ui.menu_button("Project", |ui| {
                            if ui.add(Button::new("Import models")).clicked() {
                                unimplemented!();

                                // let sender = self.sender.clone();
                                //
                                // std::thread::spawn(move || {
                                //     if let Some(paths) = FileDialog::new()
                                //         .add_filter("gltf", &["gltf", "glb"])
                                //         .set_can_create_directories(true)
                                //         .set_directory("/")
                                //         .pick_files()
                                //     {
                                //         for path in paths {
                                //             sender.send(EngineEvent::ImportModel(path)).unwrap();
                                //         }
                                //     }
                                // });
                                //
                                // ui.close();
                            }
                        });

                        ui.menu_button("Run", |ui| {
                            if ui.add(Button::new("Run game")).clicked() {
                                unimplemented!();

                                // let uuid = Uuid::new_v4().to_string();
                                // let mut temp_path = std::env::temp_dir();
                                // temp_path.push(uuid.clone());
                                //
                                // let serialized_world = SerializedWorld::from_world(&self.world, &self.engine.assets);
                                // let serialized_string = serde_json::to_string(&serialized_world).unwrap();
                                //
                                // std::fs::write(&temp_path, serialized_string).unwrap();
                                //
                                // std::process::Command::new("cargo")
                                //     .arg("run")
                                //     .arg("--package")
                                //     .arg("game")
                                //     .arg("--")
                                //     .arg("--project")
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
                .show(ctx, |_ui| {
                    // self.world.graph.show(ui);

                    // ui.add(egui::Separator::default().horizontal());

                    // ui.collapsing("Quads", |ui| {
                    //     if self.scene.quads.node_count() == 0 {
                    //         ui.label("There are no quads in the scene.");
                    //     } else {
                    //         ui::collapsing_graph(ui, &mut self.scene.quads);
                    //     }
                    // });
                });

            egui::SidePanel::right("right_panel").show(ctx, |ui| {
                ui.collapsing("Properties", |_ui| {
                    // if self.selection.len() == 1 {
                    //     let selected_node_index = self.selection[0];
                    //     let selected_node = &mut self.world.graph.graph[selected_node_index];
                    //
                    //     selected_node.local_transform.show(ui);
                    //
                    //     dbg!(&selected_node.local_transform);
                    //
                    //     ui.label(format!("Node index: {:?}", selected_node_index));
                    //
                    //     ui.separator();
                    //
                    //     ui.label("Components");
                    //
                    //     if self.world.player_spawn == Some(selected_node_index) {
                    //         ui.label("Player spawn");
                    //     }
                    //     if self.world.physics_context.colliders.contains_key(&selected_node_index) {
                    //         ui.horizontal(|ui| {
                    //             ui.label("Collider");
                    //             if ui.button("-").clicked() {
                    //                 self.world.physics_context.colliders.remove(&selected_node_index);
                    //             }
                    //         });
                    //     }
                    //
                    //     if ui.button("+").clicked() {
                    //         self.world.player_spawn = Some(selected_node_index);
                    //     }
                    // }
                });

                ui.collapsing("Debug", |_ui| {
                    unimplemented!();

                    // ui.add(
                    //     egui::Slider::new(&mut self.state.gui.debug_cube_index, 0..=self.debug_cuboids.len()).integer(),
                    // );
                    //
                    // ui.add(egui::Slider::new(&mut self.state.gui.debug_cube_opacity, 0.0..=1.0));
                    //
                    // ui.checkbox(&mut self.state.gui.render_debug_mouse_rays, "Render debug mouse rays");
                    // if ui.button("Clear lines").clicked() {
                    //     // self.engine.renderer.lines.clear();
                    //     unimplemented!();
                    // }
                });

                ui.separator();

                ui.collapsing("Background", |_ui| {
                    unimplemented!();

                    // ui.horizontal(|ui| {
                    //     // ui.selectable_value(
                    //     //     &mut self.scene.background,
                    //     //     Background::default(),
                    //     //     "Color",
                    //     // );
                    //
                    //     if ui.selectable_label(false, "HDRI").clicked() {
                    //         let sender = self.sender.clone();
                    //
                    //         std::thread::spawn(move || {
                    //             if let Some(path) = FileDialog::new()
                    //                 .set_can_create_directories(true)
                    //                 .set_directory("/")
                    //                 .pick_folder()
                    //             {
                    //                 sender.send(EngineEvent::ImportHDRIBackground(path)).unwrap();
                    //             }
                    //         });
                    //     }
                    // });
                });

                ui.collapsing("Lighting", |_ui| {
                    unimplemented!();

                    // ui.checkbox(&mut self.state.gui.render_lights, "Render lights");
                });
            });

            // Update the viewport size with the amount of space after then panels have been added
            viewport_changed.write(ViewportChanged(Viewport(Some(ctx.available_rect()))));
        });
    }
}
