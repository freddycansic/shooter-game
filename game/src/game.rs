use std::collections::HashMap;
use std::mem::Discriminant;
use std::path::PathBuf;
use std::time::Instant;
use clap::Parser;
use fxhash::FxHashMap;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use nalgebra::{Point2, Point3, Translation3, Vector2, Vector3};
use palette::Srgb;
use petgraph::data::{DataMap, DataMapMut};
use petgraph::prelude::NodeIndex;
use winit::event::{DeviceEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use common::application::Application;
use common::camera::{Camera, OrbitalCamera};
use common::collision::collidable::Intersectable;
use common::collision::colliders::sphere::Sphere;
use common::colors::Color;
use common::components::component::Component;
use common::debug;
use common::engine::Engine;
use common::input::Input;
use common::line::Line;
use common::quad::Quad;
use common::systems::renderer::Renderer;
use common::scene::graph::{NodeType, Renderable, SceneNode};
use common::scene::Scene;
use common::serde::SerializedWorld;
use crate::controllers::player::PlayerController;

struct FrameState {
    pub last_frame_end: Instant,
    pub deltatime: f64,
    pub is_moving_camera: bool,
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
            is_moving_camera: false,
        }
    }
}

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long)]
    scene: Option<String>
}

pub struct Game {
    engine: Engine,
    scene: Scene,
    state: FrameState,
    camera: OrbitalCamera,
    player: PlayerController,
    player_sphere: NodeIndex,
}

impl Application for Game {
    fn new(window: &Window, display: &Display<WindowSurface>, _event_loop: &ActiveEventLoop) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        let inner_size = window.inner_size();
        let renderer = Renderer::new(inner_size.width as f32, inner_size.height as f32, None, display).unwrap();

        let mut scene = {
            let args = Args::parse();

            let scene_path = match args.scene {
                Some(scene) => {
                    let mut path = std::env::temp_dir();
                    path.push(scene);
                    path
                },
                None => PathBuf::from("assets/game_scenes/map.json")
            };

            let serialized_scene_string = std::fs::read_to_string(scene_path).unwrap();

            serde_json::from_str::<SerializedWorld>(&serialized_scene_string).unwrap().into_scene(display).unwrap()
        };

        // scene.camera = scene.starting_camera.clone();

        // let inner_size = opengl_context.window.inner_size();
        /*scene.camera = Camera::new_fps(
            Point3::new(3.0, 0.2, 3.0),
            -Vector3::new(3.0, 0.2, 3.0).normalize(),
            inner_size.width as f32 / inner_size.height as f32,
        );*/

        let crosshair_texture = scene
            .resources
            .get_texture_handle(&PathBuf::from("assets/textures/crosshair.png"), display)
            .unwrap();

        scene.quads.0 = vec![vec![Quad::new(
            Point2::new(0.1, 0.1),
            Vector2::new(0.2, 0.2),
            crosshair_texture,
        )]];

        let state = FrameState::default();
        let input = Input::new();

        let components = scene.components();

        // TODO make this better
        let player_node = components[&std::mem::discriminant(&Component::PlayerSpawn)].first().unwrap();

        dbg!(&components);

        let player_position = scene.graph.graph.node_weight(*player_node).unwrap().local_transform.translation().vector;

        let player = PlayerController {
            position: player_position.clone(),
            velocity: Vector3::zeros(),
            node: player_node.clone(),
        };

        let sphere_renderable = Renderable {
            geometry_handle: scene
                .resources
                .get_geometry_handles(&PathBuf::from("assets/models/sphere.glb"), display)
                .unwrap()
                .into_iter()
                .next()
                .unwrap(),
            texture_handle: scene.resources.get_texture_handle(&PathBuf::from("assets/textures/gmod.jpg"), display).unwrap(),
        };

        let sphere_scene_node = SceneNode::new(NodeType::Renderable(sphere_renderable));

        let sphere_graph_node = scene.graph.add_node(sphere_scene_node);
        scene.graph.add_edge(player_node.clone(), sphere_graph_node);

        let camera = OrbitalCamera::new(Point3::from(player_position), 5.0);

        let engine = Engine {
            renderer,
            input,
            gui: None,
            scene
        }

        Self {
            renderer,
            scene,
            state,
            input,
            camera,
            components,
            player,
            player_sphere: sphere_graph_node
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
                self.engine.renderer.render(window, display);

                self.state.update_statistics();
            }
            _ => (),
        };
    }

    fn device_event(
        &mut self,
        event: DeviceEvent,
        _event_loop: &ActiveEventLoop,
        _window: &Window,
        _display: &Display<WindowSurface>,
    ) {
        self.input.process_device_event(event);
    }
}

impl Game {
    fn update(&mut self, window: &Window, _display: &Display<WindowSurface>) {
        // self.state.is_moving_camera =
        //     self.input.mouse_button_down(MouseButton::Middle) || self.input.key_down(KeyCode::Space);

        // if self.state.is_moving_camera {
        //     self.camera.update(&self.input, self.state.deltatime as f32);

        self.capture_cursor(window);
        window.set_cursor_visible(false);
        self.center_cursor(window);

        let intended_velocity = self.player.intended_velocity(&self.input, self.state.deltatime as f32);

        if intended_velocity.magnitude_squared() > 0.0 {
            let sphere = {
                let player_node = self.scene.graph.graph.node_weight_mut(self.player.node).unwrap();

                if let NodeType::Renderable(renderable) = &player_node.ty {
                    let geometry = self.scene.resources.get_geometry(renderable.geometry_handle);

                    let root_aabb = geometry.bvh.get_root_aabb();
                    dbg!(root_aabb.min, root_aabb.max);

                    let origin_world = player_node.world_transform().translation();

                    let extent = root_aabb.max - root_aabb.min;
                    let longest_side_local = extent.x.max(extent.y).max(extent.z);
                    let longest_side_world = longest_side_local * player_node.world_transform().scale();

                    Sphere::new(origin_world.vector, longest_side_world / 2.0)
                } else {
                    panic!("Player node is not a renderable type");
                }
            };

            self.scene.graph.graph.node_weight_mut(self.player_sphere).unwrap().local_transform.set_scale(sphere.radius);

            let hit = self.scene.sweep_intersect_sphere(&sphere, &intended_velocity);

            dbg!(&sphere);
            dbg!(&hit);

            self.scene.lines.clear();

            let actual_velocity = match hit {

                Some(hit) => {
                    self.scene.lines.push(Line::new(hit.point, sphere.origin, Srgb::from(palette::named::RED), 10));

                    if hit.t > 0.0 {
                        hit.t * intended_velocity * 0.90
                    } else {
                        hit.normal * 0.01
                    }
                },
                None => intended_velocity,
            };

            self.scene.lines.push(Line::new(sphere.origin, sphere.origin + actual_velocity * 100.0, Srgb::from(palette::named::RED), 10));

            self.player.position += actual_velocity;

            let player_node = self.scene.graph.graph.node_weight_mut(self.player.node).unwrap();
            player_node.local_transform.set_translation(Translation3::from(self.player.position));
        }

        self.camera.target = Point3::from(self.player.position);
        self.camera.update(&self.input, self.state.deltatime as f32);
        self.camera.update_zoom(&self.input);

        // } else {
        //     self.release_cursor(window);
        //     window.set_cursor_visible(true);
        // }

        self.input.reset_internal_state();
    }

    fn render(&mut self, _window: &Window, display: &Display<WindowSurface>) {
        let mut target = display.draw();
        {
            self.scene.render(
                &mut self.renderer,
                &self.camera.view(),
                self.camera.position(),
                display,
                &mut target,
            );
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self, _window: &Window, _display: &Display<WindowSurface>) {}
}
