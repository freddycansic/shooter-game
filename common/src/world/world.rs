use crate::collision::collidable::{RayHitNode, Sweep, SweepHitNode};
use crate::collision::colliders::sphere::Sphere;
use crate::ecs::archetype::{Archetype, Column};
use crate::ecs::component::StableId;
use crate::ecs::entity::Entity;
use crate::ecs::event::{Event, EventMessage, EventQueue};
use crate::ecs::resource::{Resource, ResourceStore};
use crate::engine::assets::{Assets, GeometryHandle, TextureHandle};
use crate::engine::renderer::Background;
use crate::light::Light;
use crate::line::Line;
use crate::maths::Ray;
use crate::serde::SerializedWorld;
use crate::world::graph::WorldGraph;
use crate::world::{PhysicsContext, QuadTree};
use common::ecs::component;
use common::ecs::owned_components::OwnedComponents;
use fxhash::FxHashMap;
use itertools::Itertools;
use petgraph::prelude::NodeIndex;
use rfd::FileDialog;
use std::collections::hash_map::Entry;

pub struct World {
    pub title: String,
    pub lines: Vec<Line>,
    pub quads: QuadTree,
    pub background: Background,
    pub graph: WorldGraph,
    pub lights: Vec<Light>,

    // Components
    pub geometries: FxHashMap<NodeIndex, GeometryHandle>,
    pub textures: FxHashMap<NodeIndex, TextureHandle>,
    pub player_spawn: Option<NodeIndex>,
    pub physics_context: PhysicsContext,

    // ECS
    pub archetypes: FxHashMap<u64, Archetype>,
    pub resources: FxHashMap<StableId, ResourceStore>,
    pub events: FxHashMap<StableId, EventQueue>,

    callbacks: FxHashMap<StableId, Vec<Box<dyn Fn(&dyn EventMessage)>>>,
}

impl World {
    pub fn spawn<T: OwnedComponents>(&mut self, components: T) -> Entity {
        self.find_exact_archetype(&T::sorted_ids()).spawn(components)
    }

    /// Finds the single archetype matching T exactly, creates it if it does not exist.
    pub fn find_exact_archetype(&mut self, ids: &[StableId]) -> &mut Archetype {
        let archetype_id = component::archetype_id(ids);

        self.archetypes.entry(archetype_id).or_insert_with_key(|id| Archetype {
            id: *id,
            entities: vec![],
            columns: ids.iter().cloned().map(Column::new_empty).collect_vec(),
        })
    }

    /// Find all archetypes which contain the query ids, returned in the order the query specifies.
    /// `query_ids` is an unsorted slice of Component ids, usually in the order specified by a `Query`
    pub fn find_matching_archetype_columns(&mut self, query_ids: &[StableId]) -> Vec<Vec<*mut Column>> {
        let mut matching_archetypes = Vec::new();

        for archetype in self.archetypes.values_mut() {
            if let Some(columns) = archetype.matching_columns(query_ids) {
                matching_archetypes.push(columns);
            }
        }

        matching_archetypes
    }

    pub fn raycast(&self, ray: &Ray, resources: &Assets) -> Option<RayHitNode> {
        self.physics_context.raycast(ray, &self.graph, resources)
    }

    pub fn spherecast(&self, sphere: &Sweep<Sphere>, resources: &Assets) -> Option<SweepHitNode> {
        self.physics_context.spherecast(sphere, &self.graph, resources)
    }

    pub fn save_as(&self, resources: &Assets) {
        let serialized_world = SerializedWorld::from_world(self, resources);

        let serialized = serde_json::to_string(&serialized_world).unwrap();

        std::thread::spawn(move || {
            if let Some(save_path) = FileDialog::new().save_file() {
                std::fs::write(save_path, serialized).unwrap();
            }
        });
    }

    pub fn register_resource<T: Resource + 'static>(&mut self, resource: T) {
        match self.resources.entry(T::ID) {
            Entry::Occupied(_) => panic!("resource already registered"),
            Entry::Vacant(entry) => entry.insert(ResourceStore::from(resource)),
        };
    }

    pub fn resource<T: Resource + 'static>(&self) -> Option<&T> {
        self.resources.get(&T::ID).and_then(|store| store.get())
    }

    pub fn resource_mut<T: Resource + 'static>(&mut self) -> Option<&mut T> {
        self.resources.get_mut(&T::ID).and_then(|store| store.get_mut())
    }

    pub fn event_queue<T: Event + 'static>(&mut self) -> &mut EventQueue {
        self.events.entry(T::ID).or_insert(EventQueue::default())
    }

    pub fn dispatch<T: Event + 'static>(&mut self, event: T) {
        if let Entry::Occupied(callbacks) = self.callbacks.entry(T::ID) {
            for callback in callbacks.get().iter() {
                callback(&event);
            }
        } else {
            log::warn!("No callbacks for Event {:?}", T::ID);
        }
    }

    pub fn add_callback<T: Event + 'static, F>(&mut self, func: F)
    where
        F: Fn(&T) + 'static,
    {
        let wrapper = Box::new(move |event: &dyn EventMessage| {
            let e = event.as_any_ref().downcast_ref::<T>().unwrap();
            func(e);
        });

        self.callbacks.entry(T::ID).or_default().push(wrapper);
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            background: Background::default(),
            quads: QuadTree::new(),
            lines: vec![],
            graph: WorldGraph::new(),
            lights: vec![],
            physics_context: PhysicsContext::new(),

            geometries: FxHashMap::default(),
            textures: FxHashMap::default(),
            player_spawn: None,
            archetypes: FxHashMap::default(),
            resources: FxHashMap::default(),
            events: FxHashMap::default(),
            callbacks: FxHashMap::default(),
        }
    }
}
