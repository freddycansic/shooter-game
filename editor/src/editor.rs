use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use common::debug::Cuboid;
use common::maths::{Ray, Transform};
use common::serde::SerializedWorld;
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
use common::ecs::entity::Entity;
use common::ecs::subsystem::Subsystem;
use common::ecs::system_parameters::commands::Commands;
use common::ecs::system_parameters::event::{EventReader, EventWriter};
use common::ecs::system_parameters::query::Query;
use common::ecs::system_parameters::res::{Res, ResMut};
use common::engine::assets::{Assets, GeometryHandle, TextureHandle};
use common::engine::engine::Engine;
use common::engine::input::Input;
use common::engine::physics;
use common::engine::physics::ColliderSet;
use common::engine::renderer::{Background, Renderer, Viewport, ViewportChanged};
use common::engine::scheduler::{Scheduler, Stage};
use common::executor::{CommandExecutor, RuntimeContext};
use common::gui::{Gui, GuiState};
use common::light::Light;
use common::line::Line;
use common::maths::transform::WorldTransform;
use common::subsystems::frame_timing::{FrameTiming, WinitNewEvents};
use common::subsystems::window_size::WindowSize;
use common::world::World;
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
    world: World,
}

impl Application for Editor {
    fn new(context: &RuntimeContext) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        // TODO deferred rendering https://learnopengl.com/Advanced-Lighting/Deferred-Shading

        let mut world = World::default();

        let engine = Engine::new();

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
        };

        editor.register_subsystem_with_context::<OrbitalCamera>(context);
        editor.register_subsystem_with_context::<Gui>(context);

        editor
            .engine
            .scheduler
            .register_continuous(Self::detect_viewport_click, Stage::Pre);
        editor
            .engine
            .scheduler
            .register_triggered::<ViewportClick, _, _>(Self::selection_stuff, Stage::Main);
        editor.engine.scheduler.register_continuous(Self::render, Stage::Render);

        // TODO temporary, should make selection subsystem
        editor.world.register_resource(Selection(vec![]));
        
        editor
    }

    fn run(&mut self, mut context: RuntimeContext) {
        let input = self.world.resource::<Input>().unwrap();

        if input.key_pressed(KeyCode::Escape) {
            context.exit();
        }

        // TODO turn this into ECS systems
        self.update(context.display);

        self.engine.scheduler.run(&mut self.world, &mut context);
    }

    fn window_event(
        &mut self,
        event: WindowEvent,
        _event_loop: &ActiveEventLoop,
        window: &Window,
        _display: &Display<WindowSurface>,
    ) {
    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }

    fn scheduler(&mut self) -> &mut Scheduler {
        &mut self.engine.scheduler
    }
}

impl Editor {
    fn render(
        window_size: Res<WindowSize>,
        mut renderer: ResMut<Renderer>,
        viewport: Res<Viewport>,
        commands: Commands,
        background: Res<Background>,
        selection: Res<Selection>,
        assets: Res<Assets>,
        camera: Res<OrbitalCamera>,
        mut gui: ResMut<Gui>,
        mut geometry: Query<(&GeometryHandle, &WorldTransform, Option<&TextureHandle>)>,
    ) {
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        let mut target = commands.display().draw();
        {
            renderer.render_world(
                geometry.iter(),
                &*camera,
                &*assets,
                &selection.0,
                commands.display(),
                &*viewport,
                &*background,
                &mut target,
            );

            gui.0.paint(commands.display(), &mut target);
        }
        target.finish().unwrap();
    }

    // TODO turn all this into ECS event stuff
    fn update(&mut self, display: &Display<WindowSurface>) {
        // let events = self.receiver.try_iter().collect_vec();
        //
        // // TODO turn these into executor events
        // for engine_event in events.into_iter() {
        //     match engine_event {
        //         EngineEvent::LoadProject(serialized_project) => {
        //             // At the moment this just loads a world
        //             // In the future it might be necessary to have multiple worlds in one project.
        //             let serialized_world = serde_json::from_str::<SerializedWorld>(&serialized_project).unwrap();
        //
        //             self.world = serialized_world.into_world(display, &mut self.engine.assets).unwrap();
        //         }
        //         EngineEvent::ImportModel(model_path) => self.import_model(model_path.as_path(), display).unwrap(),
        //         EngineEvent::ImportHDRIBackground(hdri_directory_path) => {
        //             self.world.background = Background::HDRI(
        //                 self.engine
        //                     .assets
        //                     .get_cubemap_handle(&hdri_directory_path, display)
        //                     .unwrap(),
        //             )
        //         }
        //     }
        // }

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
        let mouse_in_viewport = Self::is_mouse_in_viewport(&input, &viewport);
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

        viewport
            .0
            .is_some_and(|viewport| viewport.contains(Pos2::new(mouse_position.x as f32, mouse_position.y as f32)))
    }

    fn selection_stuff(
        gui_state: Res<GuiState>,
        mut viewport_click: EventReader<ViewportClick>,
        assets: Res<Assets>,
        mut selection: ResMut<Selection>,
        mut colliders: Query<(&ColliderSet, &WorldTransform)>,
    ) {
        // let mouse_ray = viewport_click.read().next().unwrap().mouse_ray;

        // let intersection = physics::raycast(&mouse_ray, colliders.iter(), &assets);
        //
        // if gui_state.render_debug_mouse_rays {
        //     self.world.lines.push(Line::new(
        //         ray.origin,
        //         ray.origin + ray.direction() * 1000.0,
        //         if intersection.is_some() {
        //             Srgb::new(0.0, 1.0, 0.0)
        //         } else {
        //             Srgb::new(1.0, 0.0, 0.0)
        //         },
        //         2,
        //     ));
        // }
        //
        // selection = match intersection {
        //     Some(hit) => vec![hit.node],
        //     None => vec![],
        // };
    }

    /// Load a models and create an instance of it in the world
    fn import_model(&mut self, path: &Path, display: &Display<WindowSurface>) -> color_eyre::Result<()> {
        unimplemented!();

        // let handles = self.engine.assets.get_geometry_handles(path, Some(display))?;
        //
        // let group_node = self.world.graph.add_root_node(WorldNode::default());
        //
        // for geometry_handle in handles {
        //     let world_node = WorldNode::default();
        //     let world_graph_node = self.world.graph.add_node(world_node);
        //     self.world.graph.add_edge(group_node, world_graph_node);
        //
        //     self.world
        //         .physics_context
        //         .colliders
        //         .insert(world_graph_node, ColliderSet::from(geometry_handle));
        //     self.world.geometries.insert(world_graph_node, geometry_handle);
        // }
        //
        // Ok(())
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
