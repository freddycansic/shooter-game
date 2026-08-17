use clap::Parser;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use nalgebra::{Point2, Point3, Translation3, Vector2, Vector3};
use std::path::PathBuf;
use std::time::Instant;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::controllers::player::PlayerController;
use common::camera::{Camera, OrbitalCamera};
use common::collision::collidable::Sweep;
use common::collision::colliders::sphere::Sphere;
use common::debug;
use common::engine::engine::Engine;
use common::quad::Quad;
use common::runtime::application::Application;
use common::serde::SerializedWorld;
use common::world::World;

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
    project: Option<String>,
}

pub struct Game {
    engine: Engine,
    world: World,
    state: FrameState,
    camera: OrbitalCamera,
    player: PlayerController,
}

impl Application for Game {
    fn new(window: &Window, display: &Display<WindowSurface>, event_loop: &ActiveEventLoop) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        let mut engine = Engine::new(None /* full size */, display, window, event_loop);

        engine.assets.initialise_default_texture(display).unwrap();

        let mut world = {
            let args = Args::parse();

            let project_path = match args.project {
                Some(project) => {
                    let mut path = std::env::temp_dir();
                    path.push(project);
                    path
                }
                None => PathBuf::from("assets/projects/map.json"),
            };

            let serialized_world_string = std::fs::read_to_string(project_path).unwrap();

            serde_json::from_str::<SerializedWorld>(&serialized_world_string)
                .unwrap()
                .into_world(display, &mut engine.assets)
                .unwrap()
        };

        // scene.camera = scene.starting_camera.clone();

        // let inner_size = opengl_context.window.inner_size();
        /*scene.camera = Camera::new_fps(
            Point3::new(3.0, 0.2, 3.0),
            -Vector3::new(3.0, 0.2, 3.0).normalize(),
            inner_size.width as f32 / inner_size.height as f32,
        );*/

        let crosshair_texture = engine
            .assets
            .get_texture_handle(&PathBuf::from("assets/textures/crosshair.png"), display)
            .unwrap();

        world.quads.0 = vec![vec![Quad::new(
            Point2::new(0.1, 0.1),
            Vector2::new(0.2, 0.2),
            crosshair_texture,
        )]];

        let state = FrameState::default();

        let player = PlayerController::initialise(&mut world, &mut engine.assets, display);

        let inner_size = window.inner_size();
        let camera = OrbitalCamera::new(
            /* TODO */ Point3::origin(),
            5.0,
            inner_size.width as f32,
            inner_size.height as f32,
        );

        Self {
            engine,
            world,
            state,
            camera,
            player,
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
    }

    fn device_event(
        &mut self,
        event: DeviceEvent,
        _event_loop: &ActiveEventLoop,
        _window: &Window,
        _display: &Display<WindowSurface>,
    ) {
        self.engine.input.process_device_event(event);
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

        self.player.update_velocity_on_input(&self.engine.input);
        let gravity = Vector3::new(0.0, -9.8, 0.0) * 0.1;
        self.player.velocity += gravity * self.state.deltatime as f32;

        if self.player.velocity.magnitude_squared() > 0.0 {
            let player_displacement = self.player.velocity * self.state.deltatime as f32;

            let graph_node = self.world.graph.graph.node_weight(self.player.node).unwrap();

            let world_sphere = Sphere {
                origin: graph_node.world_transform().translation().vector.into(),
                radius: 5.0,
                // TODO
                // let collider_set = self.world.physics_context.colliders.get(&self.player.node).unwrap();
                // let collider_set = ColliderSet::narrow_only(Collider::Sphere(Sphere::new(Point3::origin(), 5.0)));
                //
                //
                // if let Collider::Sphere(sphere) = &collider_set.narrow {
                //     log::debug!("Local sphere collider {:?}", sphere);
                //
                //     let world_origin = graph_node.world_transform().matrix().transform_point(&sphere.origin);
                //     let world_scale = graph_node.world_transform().scale();
                //     let max_scale = world_scale.x.max(world_scale.y).max(world_scale.z);
                //     let world_radius = sphere.radius * max_scale;
                //
                //     Sphere::new(world_origin, world_radius)
                // } else {
                //     panic!()
                // }
            };

            log::debug!("World sphere collider {:?}", &world_sphere);

            let hit = self
                .world
                .spherecast(&Sweep::new(world_sphere, player_displacement), &self.engine.assets);

            log::debug!("hit {:?}", &hit);

            let actual_displacement = match hit {
                Some(hit) => {
                    let normal = hit.hit.normal;
                    self.player.velocity -= self.player.velocity.dot(&normal) * normal;

                    hit.hit.t * player_displacement
                }
                None => player_displacement,
            };

            self.player.position += actual_displacement;
            self.player.velocity *= 0.9;
        }

        //
        let player_node = self.world.graph.graph.node_weight_mut(self.player.node).unwrap();
        player_node
            .local_transform
            .set_translation(Translation3::from(self.player.position));
        // }

        // self.camera.target = Point3::from(self.player.position);
        self.camera.update(&self.engine.input, self.state.deltatime as f32);
        self.camera.update_zoom(&self.engine.input);

        // } else {
        //     self.release_cursor(window);
        //     window.set_cursor_visible(true);
        // }

        self.engine.input.reset_internal_state();
    }

    fn render(&mut self, _window: &Window, display: &Display<WindowSurface>) {
        let mut target = display.draw();
        {
            self.world.graph.calculate_world_matrices();
            self.engine.renderer.render_world(
                &self.world,
                &self.camera,
                &self.engine.assets,
                &[],
                display,
                &mut target,
            );
        }
        target.finish().unwrap();
    }

    fn render_gui(&mut self, _window: &Window, _display: &Display<WindowSurface>) {}
}
