use common::engine::input::Input;
use common::engine::renderer::Renderable;
use common::engine::resources::Resources;
use common::world::{World, WorldNode};
use glium::glutin::surface::WindowSurface;
use glium::Display;
use nalgebra::{Point3, Vector3};
use petgraph::prelude::NodeIndex;
use std::path::PathBuf;
use winit::keyboard::KeyCode;

pub struct PlayerController {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub node: NodeIndex,

    pub collider_node: NodeIndex,
}

impl PlayerController {
    pub fn initialise(world: &mut World, resources: &mut Resources, display: &Display<WindowSurface>) -> Self {
        let player_node = world.player_spawn.unwrap();

        let player_position = world
            .graph
            .graph
            .node_weight(player_node)
            .unwrap()
            .local_transform
            .translation();

        // Set up the collider for rendering
        let collider_world_node = WorldNode::default();
        let collider_graph_node = world.graph.add_node(collider_world_node);
        world.graph.add_edge(player_node, collider_graph_node);

        let collider_renderable = Renderable {
            geometry_handle: *resources
                .get_geometry_handles(&PathBuf::from("assets/models/sphere.glb"), Some(display))
                .unwrap()
                .first()
                .unwrap(),
            texture_handle: resources.default_texture().unwrap(),
        };
        world.renderables.insert(collider_graph_node, collider_renderable);

        let player = PlayerController {
            position: player_position.vector.into(),
            velocity: Vector3::zeros(),
            node: player_node.clone(),
            collider_node: collider_graph_node,
        };

        // TODO
        // let player_collider = Sphere::new(player.position.clone(), 5.0);
        // let player_collider_set = ColliderSet::narrow_only(Collider::Sphere(player_collider));
        //
        // world.physics_context.colliders.insert(player.node, player_collider_set);

        player
    }

    pub fn update_velocity_on_input(&mut self, input: &Input) {
        // TODO
        let forward_direction = Vector3::z_axis().into_inner();
        let left_direction = Vector3::x_axis().into_inner();
        let speed = 1.0;

        let mut intended_velocity = Vector3::new(0.0, 0.0, 0.0);

        if input.key_down(KeyCode::KeyW) {
            intended_velocity += forward_direction * speed;
        }

        if input.key_down(KeyCode::KeyS) {
            intended_velocity -= forward_direction * speed;
        }

        if input.key_down(KeyCode::KeyA) {
            intended_velocity -= left_direction * speed;
        }

        if input.key_down(KeyCode::KeyD) {
            intended_velocity += left_direction * speed;
        }

        self.velocity += intended_velocity;
    }
}
