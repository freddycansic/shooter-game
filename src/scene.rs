use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use vulkano::command_buffer::RecordingCommandBuffer;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::GraphicsPipeline;

use crate::camera::Camera;
use crate::context::Allocators;
use crate::model::{Model, ModelInstance};
use crate::texture::Texture;

pub struct Scene {
    pub model_instances: Vec<ModelInstance>,
    pub camera: Camera,
    models: HashMap<String, Arc<Model>>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            model_instances: vec![],
            models: HashMap::new(),
            camera,
        }
    }

    /// Load a model and create an instance of it in the scene
    pub fn import_model(&mut self, path: &str, memory_allocator: Arc<StandardMemoryAllocator>) {
        let model = self.load_model(path, memory_allocator);

        self.model_instances.push(ModelInstance::from(model));
    }

    /// Load a model into the cache
    pub fn load_model(
        &mut self,
        path: &str,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Arc<Model> {
        self.models
            .entry(path.to_owned())
            .or_insert(Model::load(path, memory_allocator).unwrap())
            .clone()
    }

    pub fn model_is_loaded(&self, path: &str) -> bool {
        self.models.contains_key(path)
    }

    pub fn render(
        &self,
        builder: &mut RecordingCommandBuffer,
        allocators: &Allocators,
        pipeline: Arc<GraphicsPipeline>,
        // TODO temporary
        texture: &Texture,
    ) {
        // build per-frame uniforms

        // draw each primitive
        self.model_instances.iter().for_each(|model_instance| {
            model_instance.render(builder, allocators, pipeline.clone(), texture)
        })
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera::default())
    }
}
