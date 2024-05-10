use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::Arc;

use cgmath::{Matrix, Matrix4, SquareMatrix};
use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::{
    implement_vertex, uniform, Display, DrawParameters, Frame, Program, Surface, VertexBuffer,
};
use itertools::Itertools;
use serde::de::{MapAccess, Visitor};
use serde::ser::{SerializeMap, SerializeStruct, SerializeTuple};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::camera::Camera;
use crate::maths;
use crate::model::{Model, ModelInstance, Transform};

pub struct Scene {
    pub model_instances: Vec<ModelInstance>,
    pub camera: Camera,
    pub title: String,
    models: HashMap<String, Arc<Model>>,
}

impl Scene {
    pub fn new(title: String, camera: Camera) -> Self {
        Self {
            model_instances: vec![],
            models: HashMap::new(),
            title,
            camera,
        }
    }

    pub fn deserialize(serialised: &str, display: &Display<WindowSurface>) -> Result<Self> {
        let unloaded_scene = serde_json::from_str::<UnloadedScene>(serialised)?;

        let mut scene = Scene::new(unloaded_scene.title, unloaded_scene.camera);

        for (path, transforms) in unloaded_scene.model_paths_to_transforms.iter() {
            let model = scene.load_model(path, display)?;
            for transform in transforms {
                scene.model_instances.push(ModelInstance {
                    model: model.clone(),
                    transform: transform.clone(),
                });
            }
        }

        Ok(scene)
    }

    /// Load a model and create an instance of it in the scene
    pub fn import_model(&mut self, path: &str, display: &Display<WindowSurface>) -> Result<()> {
        let model = self.load_model(path, display)?;

        self.model_instances.push(ModelInstance::from(model));

        Ok(())
    }

    /// Load a model into the cache
    pub fn load_model(
        &mut self,
        path: &str,
        display: &Display<WindowSurface>,
    ) -> Result<Arc<Model>> {
        Ok(self
            .models
            .entry(path.to_owned())
            .or_insert(Model::load(path, display)?)
            .clone())
    }

    pub fn model_is_loaded(&self, path: &str) -> bool {
        self.models.contains_key(path)
    }

    pub fn render(&self, program: &Program, display: &Display<WindowSurface>, target: &mut Frame) {
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
                    target
                        .draw(
                            (
                                &primitive.vertex_buffer,
                                instance_buffer.per_instance().unwrap(),
                            ),
                            &primitive.index_buffer,
                            program,
                            &uniforms,
                            &DrawParameters::default(),
                        )
                        .unwrap();
                }
            }
        }
    }

    fn build_instance_map(&self) -> HashMap<Arc<Model>, Vec<Instance>> {
        let mut instance_map = HashMap::<Arc<Model>, Vec<Instance>>::new();

        for model_instance in self.model_instances.iter() {
            let transform_matrix = Matrix4::from(model_instance.transform.clone());

            let instance = Instance {
                transform: <[[f32; 4]; 4]>::from(transform_matrix),
                transform_normal: <[[f32; 4]; 4]>::from(
                    transform_matrix.invert().unwrap().transpose(),
                ),
            };

            instance_map
                .entry(model_instance.model.clone())
                .or_insert(vec![instance])
                .push(instance);
        }

        instance_map
    }

    fn build_instance_buffers(
        &self,
        display: &Display<WindowSurface>,
    ) -> Vec<(Arc<Model>, VertexBuffer<Instance>)> {
        let instance_map = self.build_instance_map();

        instance_map
            .into_iter()
            .map(|(model, instances)| (model, VertexBuffer::new(display, &instances).unwrap()))
            .collect_vec()
    }
}

impl Serialize for Scene {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut instance_map = HashMap::<String, Vec<Transform>>::new();

        for model_instance in self.model_instances.iter() {
            instance_map
                .entry(model_instance.model.path.clone())
                .or_insert(vec![model_instance.transform.clone()])
                .push(model_instance.transform.clone());
        }

        let mut s = serializer.serialize_struct("Scene", 2)?;
        s.serialize_field("model_instances", &instance_map)?;
        s.serialize_field("camera", &self.camera)?;

        s.end()
    }
}

struct UnloadedScene {
    pub camera: Camera,
    pub title: String,
    pub model_paths_to_transforms: HashMap<String, Vec<Transform>>,
}

impl<'de> Deserialize<'de> for UnloadedScene {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "UnloadedScene",
            &["model_paths_to_transforms", "camera", "title"],
            UnloadedSceneVisitor,
        )
    }
}

struct UnloadedSceneVisitor;

impl<'de> Visitor<'de> for UnloadedSceneVisitor {
    type Value = UnloadedScene;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a valid Scene")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut unloaded_scene = UnloadedScene {
            camera: Camera::default(),
            title: String::new(),
            model_paths_to_transforms: HashMap::new(),
        };

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "model_instances" => {
                    unloaded_scene.model_paths_to_transforms =
                        map.next_value::<HashMap<String, Vec<Transform>>>()?
                }
                "camera" => unloaded_scene.camera = map.next_value::<Camera>()?,
                "title" => unloaded_scene.title = map.next_value::<String>()?,
                _ => {
                    return Err(de::Error::unknown_field(
                        key.as_str(),
                        &["model_paths_to_transforms", "camera", "title"],
                    ))
                }
            };
        }

        Ok(unloaded_scene)
    }
}

#[derive(Copy, Clone)]
struct Instance {
    transform: [[f32; 4]; 4],
    transform_normal: [[f32; 4]; 4],
}
implement_vertex!(Instance, transform, transform_normal);

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled".to_owned(), Camera::default())
    }
}
