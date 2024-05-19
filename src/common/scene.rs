use std::collections::HashMap;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};

use cgmath::{Point3, Vector3};
use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::{
    Display, Frame, Surface,
};
use palette::Srgb;
use rfd::FileDialog;
use serde::de::{MapAccess, Visitor};
use serde::ser::{SerializeMap, SerializeStruct, SerializeTuple};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use winit::dpi::PhysicalSize;

use crate::camera::Camera;
use crate::line::{Line};
use crate::model::{ModelInstance, Transform};
use crate::renderer::Renderer;
use crate::{model, texture};

pub struct Scene {
    pub camera: Camera, // the last camera state when editing the scene
    pub title: String,
    pub starting_camera: Camera, // the camera state to be used when starting the game

    pub model_instances: Vec<ModelInstance>,
    pub lines: Vec<Line>,
}

impl Scene {
    pub fn new(title: &str, camera: Camera) -> Self {
        let lines = vec![
            Line::new(
                Point3::new(-1000.0, 0.0, 0.0),
                Point3::new(1000.0, 0.0, 0.0),
                Srgb::from(palette::named::RED),
                2,
            ),
            Line::new(
                Point3::new(0.0, -1000.0, 0.0),
                Point3::new(0.0, 1000.0, 0.0),
                Srgb::from(palette::named::GREEN),
                2,
            ),
            Line::new(
                Point3::new(0.0, 0.0, -1000.0),
                Point3::new(0.0, 0.0, 1000.0),
                Srgb::from(palette::named::BLUE),
                2,
            ),
        ];

        Self {
            model_instances: vec![],
            lines,
            title: title.to_owned(),
            starting_camera: Camera::new_fps(
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
                1920.0 / 1008.0,
            ),
            camera,
        }
    }

    pub fn deserialize(
        serialised: &str,
        display: &Display<WindowSurface>,
        inner_size: PhysicalSize<u32>,
    ) -> Result<Self> {
        let mut unloaded_scene = serde_json::from_str::<UnloadedScene>(serialised)?;

        unloaded_scene
            .camera
            .set_aspect_ratio(inner_size.width as f32 / inner_size.height as f32);

        let mut scene = Scene::new(&unloaded_scene.title, unloaded_scene.camera);

        for (model_path, textures_to_transforms) in unloaded_scene.models_to_transforms.into_iter()
        {
            let model = model::load(PathBuf::from(model_path.clone()), display)?;

            for (texture_path, transforms) in textures_to_transforms.iter() {
                let texture = if !texture_path.is_empty() {
                    Some(texture::load(PathBuf::from(texture_path), display)?)
                } else {
                    None
                };

                for transform in transforms {
                    scene.model_instances.push(ModelInstance {
                        model: model.clone(),
                        texture: texture.clone(),
                        transform: transform.clone(),
                    });
                }
            }
        }

        Ok(scene)
    }

    pub fn save_as(&self) {
        let serialized = serde_json::to_string(self).unwrap();

        std::thread::spawn(move || {
            if let Some(save_path) = FileDialog::new().save_file() {
                std::fs::write(save_path, serialized).unwrap();
            }
        });
    }

    /// Load a model and create an instance of it in the scene
    pub fn import_model(&mut self, path: &Path, display: &Display<WindowSurface>) -> Result<()> {
        let model = model::load(path.to_path_buf(), display)?;

        self.model_instances.push(ModelInstance::from(model));

        Ok(())
    }

    pub fn render(
        &mut self,
        renderer: &mut Renderer,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        target.clear_color_and_depth((0.01, 0.01, 0.01, 1.0), 1.0);

        renderer.render_model_instances(&self.model_instances, &self.camera, display, target);
        renderer.render_lines(&self.lines, &self.camera, display, target);
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled", Camera::default())
    }
}

impl Serialize for Scene {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut instance_map = HashMap::<String, HashMap<String, Vec<Transform>>>::new();

        for model_instance in self.model_instances.iter() {
            let texture_path = model_instance
                .texture
                .as_ref()
                .map(|texture| texture.path.clone().to_string_lossy().to_string())
                .unwrap_or(String::new());

            // Code is perfectly readable
            instance_map
                .entry(
                    model_instance
                        .model
                        .path
                        .clone()
                        .to_string_lossy()
                        .to_string(),
                )
                .or_insert(HashMap::from([(
                    texture_path.clone(),
                    vec![model_instance.transform.clone()],
                )]))
                .entry(texture_path.to_string())
                .or_insert(vec![model_instance.transform.clone()])
                .push(model_instance.transform.clone());
        }

        let mut s = serializer.serialize_struct("Scene", 2)?;
        s.serialize_field("model_instances", &instance_map)?;
        s.serialize_field("camera", &self.camera)?;
        s.serialize_field("starting_camera", &self.starting_camera)?;
        s.serialize_field("title", &self.title)?;

        s.end()
    }
}

/// A Scene where the models are represented as Strings instead of Models
struct UnloadedScene {
    pub camera: Camera,
    pub starting_camera: Camera,
    pub title: String,
    pub models_to_transforms: HashMap<String, HashMap<String, Vec<Transform>>>,
}

impl<'de> Deserialize<'de> for UnloadedScene {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "UnloadedScene",
            &["models_to_transforms", "camera", "starting_camera", "title"],
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
            starting_camera: Camera::default(),
            title: String::new(),
            models_to_transforms: HashMap::new(),
        };

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "model_instances" => {
                    unloaded_scene.models_to_transforms =
                        map.next_value::<HashMap<String, HashMap<String, Vec<Transform>>>>()?
                }
                "camera" => unloaded_scene.camera = map.next_value::<Camera>()?,
                "starting_camera" => unloaded_scene.starting_camera = map.next_value::<Camera>()?,
                "title" => unloaded_scene.title = map.next_value::<String>()?,
                _ => {
                    return Err(de::Error::unknown_field(
                        key.as_str(),
                        &["models_to_transforms", "camera", "title"],
                    ))
                }
            };
        }

        Ok(unloaded_scene)
    }
}
