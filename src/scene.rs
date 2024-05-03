use std::collections::HashMap;
use std::sync::Arc;
use cgmath::{Matrix, Matrix4, SquareMatrix};
use glium::{Display, DrawParameters, Frame, implement_vertex, Program, Surface, uniform, VertexBuffer};
use glium::glutin::surface::WindowSurface;
use glium::uniforms::{AsUniformValue, EmptyUniforms, UniformsStorage};
use itertools::Itertools;

use crate::camera::Camera;
use crate::maths;
use crate::model::{Model, ModelInstance};

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
    pub fn import_model(&mut self, path: &str, display: &Display<WindowSurface>) {
        let model = self.load_model(path, display);

        self.model_instances.push(ModelInstance::from(model));
    }

    /// Load a model into the cache
    pub fn load_model(
        &mut self,
        path: &str,
        display: &Display<WindowSurface>
    ) -> Arc<Model> {
        self.models
            .entry(path.to_owned())
            .or_insert(Model::load(path, &display).unwrap())
            .clone()
    }

    pub fn model_is_loaded(&self, path: &str) -> bool {
        self.models.contains_key(path)
    }

    pub fn render(
        &self,
        program: &Program,
        display: &Display<WindowSurface>,
        target: &mut Frame
    ) {
        let instance_buffers = self.build_instance_buffers(display);

            target.clear_color(1.0, 0.0, 0.0, 1.0);

            let uniforms = uniform! {
                projection: maths::raw_matrix(self.camera.projection),
                view: maths::raw_matrix(self.camera.view_matrix()),
                camera_position: <[f32; 3]>::from(self.camera.position)
            };

            for (model, instance_buffer) in instance_buffers {
                for mesh in model.meshes.iter() {
                    for primitive in mesh.primitives.iter() {
                        target.draw(
                            (
                                &primitive.vertex_buffer,
                                instance_buffer.per_instance().unwrap()
                            ),
                            &primitive.index_buffer,
                            program,
                            &uniforms,
                            &DrawParameters::default()
                        ).unwrap();
                    }
                }
            }
    }

    fn build_instance_buffers(&self, display: &Display<WindowSurface>) -> Vec<(Arc<Model>, VertexBuffer<Instance>)>{
        let mut instance_buffers = HashMap::<Arc<Model>, Vec<Instance>>::new();
        for model_instance in self.model_instances.iter() {
            let transform_matrix = Matrix4::from(model_instance.transform.clone());

            let instance = Instance {
                transform: <[[f32; 4]; 4]>::from(transform_matrix),
                transform_normal: <[[f32; 4]; 4]>::from(transform_matrix.invert().unwrap().transpose())
            };

            instance_buffers.entry(model_instance.model.clone()).or_insert(vec![instance]).push(instance);
        }

        instance_buffers.into_iter().map(|(model, instances)| {
            (model, VertexBuffer::new(display, &instances).unwrap())
        }).collect_vec()
    }
}

#[derive(Copy, Clone)]
struct Instance {
    transform: [[f32; 4]; 4],
    transform_normal: [[f32; 4]; 4]
}
implement_vertex!(Instance, transform, transform_normal);

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera::default())
    }
}
