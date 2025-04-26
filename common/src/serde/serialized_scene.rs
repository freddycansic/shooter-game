use std::{convert::identity, ops::Deref, sync::Arc};

use color_eyre::eyre::Result;
use fxhash::{FxBuildHasher, FxHashMap};
use glium::{
    Display, IndexBuffer, VertexBuffer, glutin::surface::WindowSurface, index::PrimitiveType,
};
use petgraph::prelude::StableDiGraph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    camera::FpsCamera,
    light::Light,
    models::{Model, ModelInstance, Primitive},
    physics::Physics,
    quad::Quad,
    scene::{Background, Scene},
    terrain::Terrain,
};

use super::{SerializedModel, SerializedModelInstance};

#[derive(Serialize, Deserialize)]
pub struct SerializedScene {
    pub title: String,
    pub camera: FpsCamera,
    pub serialized_model_instance_graph: StableDiGraph<SerializedModelInstance, ()>,
    pub background: Background,
    pub lights: Vec<Light>,
    // pub terrain: Option<Terrain>,
    pub quads: StableDiGraph<Quad, ()>,
    pub physics: Physics,
    pub serialized_models: FxHashMap<Uuid, SerializedModel>,
    // pub serialized_materials: FxHashMap<Uuid, SerializedMa
}

impl From<&Scene> for SerializedScene {
    fn from(value: &Scene) -> Self {
        let hasher = FxBuildHasher::new();

        let mut serialized_models = FxHashMap::<Uuid, SerializedModel>::with_hasher(hasher.clone());
        // let mut serialized_materials = FxHashMap::with_hasher(hasher.clone());

        let serialized_model_instance_graph = value.graph.map(
            |_, model_instance| {
                serialized_models.insert(
                    model_instance.model.uuid,
                    SerializedModel::from(model_instance.model.clone()),
                );

                SerializedModelInstance::from(model_instance.clone())
            },
            |_, edge| *edge,
        );

        Self {
            title: value.title.clone(),
            camera: value.camera.clone(),
            serialized_model_instance_graph,
            background: value.background.clone(),
            lights: value.lights.clone(),
            // terrain: value.terrain.clone(),
            quads: value.quads.clone(),
            physics: value.physics.clone(),
            serialized_models,
        }
    }
}

impl SerializedScene {
    pub fn into_scene(self, display: &Display<WindowSurface>) -> Result<Scene> {
        let models = self
            .serialized_models
            .into_iter()
            .map(|(uuid, serialized_model)| {
                let model = Arc::new(Model {
                    name: serialized_model.name,
                    uuid,
                    primitives: serialized_model
                        .primitives
                        .into_iter()
                        .map(|serialized_primitive| {
                            let vertex_buffer =
                                VertexBuffer::new(display, &serialized_primitive.vertices)?;
                            let index_buffer = IndexBuffer::new(
                                display,
                                PrimitiveType::TrianglesList,
                                &serialized_primitive.indices,
                            )?;

                            Ok(Primitive {
                                vertices: serialized_primitive.vertices.clone(),
                                indices: serialized_primitive.indices.clone(),
                                vertex_buffer,
                                index_buffer,
                            })
                        })
                        .collect::<Result<Vec<Primitive>>>()?, // Collect and propagate errors as Report
                });

                Ok((uuid, model))
            })
            .collect::<Result<FxHashMap<Uuid, Arc<Model>>>>()?;

        let graph = self.serialized_model_instance_graph.map(
            |_, serialized_model_instance| ModelInstance {
                material: None,
                name: serialized_model_instance.name.clone(),
                transform: serialized_model_instance.transform.clone(),
                model: models
                    .get(&serialized_model_instance.model)
                    .expect("Did not load model from serialized models")
                    .clone(),
                selected: false,
            },
            |_, edge| *edge,
        );

        Ok(Scene {
            title: self.title,
            camera: self.camera,
            graph,
            background: self.background,
            lights: self.lights,
            // terrain: self.terrain,
            quads: self.quads,
            physics: self.physics,
            lines: vec![],
        })
    }
}
