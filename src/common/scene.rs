use std::collections::HashMap;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cgmath::{Matrix, Matrix4, Point3, SquareMatrix, Vector3};
use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::index::{NoIndices, PrimitiveType};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerBehavior};
use glium::{
    implement_vertex, uniform, Depth, DepthTest, Display, DrawParameters, Frame, Program, Surface,
    VertexBuffer,
};
use itertools::Itertools;
use palette::Srgb;
use rfd::FileDialog;
use serde::de::{MapAccess, Visitor};
use serde::ser::{SerializeMap, SerializeStruct, SerializeTuple};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use winit::dpi::PhysicalSize;

use crate::camera::Camera;
use crate::line::{Line, LinePoint};
use crate::model::{Model, ModelInstance, Transform};
use crate::texture::Texture;
use crate::{context, maths, model, texture};

pub struct Scene {
    pub camera: Camera, // the last camera state when editing the scene
    pub title: String,
    pub starting_camera: Camera, // the camera state to be used when starting the game

    pub model_instances: Vec<ModelInstance>,
    pub lines: Vec<Line>,

    model_program: Program,
    lines_program: Program,

    line_vertex_buffers: Option<Vec<(u8, VertexBuffer<LinePoint>)>>,
}

impl Scene {
    pub fn new(title: &str, camera: Camera, display: &Display<WindowSurface>) -> Result<Self> {
        let model_program = context::new_program(
            "assets/shaders/default/default.vert",
            "assets/shaders/default/default.frag",
            None,
            display,
        )?;

        let lines_program = context::new_program(
            "assets/shaders/line/line.vert",
            "assets/shaders/line/line.frag",
            None,
            display,
        )?;

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

        Ok(Self {
            model_instances: vec![],
            lines,
            model_program,
            lines_program,
            title: title.to_owned(),
            starting_camera: Camera::new_fps(
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
                1920.0 / 1008.0,
            ),
            camera,
            line_vertex_buffers: None,
        })
    }

    pub fn new_untitled(display: &Display<WindowSurface>) -> Result<Self> {
        Self::new("Untitled", Camera::default(), display)
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

        let mut scene = Scene::new(&unloaded_scene.title, unloaded_scene.camera, display)?;

        for (model_path, textures_to_transforms) in unloaded_scene.models_to_transforms.into_iter()
        {
            let model = model::load(PathBuf::from(model_path.clone()), display)?;

            for (texture_path, transforms) in textures_to_transforms.iter() {
                dbg!(transforms.len());

                let texture = if !texture_path.is_empty() {
                    Some(texture::load(PathBuf::from(texture_path), display).expect(
                        "Could not load a texture path from the scene //TODO make this a normal error",
                    ))
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

    pub fn render(&mut self, display: &Display<WindowSurface>, target: &mut Frame) {
        target.clear_color_and_depth((0.01, 0.01, 0.01, 1.0), 1.0);

        self.render_models(display, target);
        self.render_lines(display, target);
    }

    fn render_models(&self, display: &Display<WindowSurface>, target: &mut Frame) {
        let instance_buffers = self.build_instance_buffers(display);

        let vp = maths::raw_matrix(self.camera.view_projection);
        let camera_position = <[f32; 3]>::from(self.camera.position);

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        for ((model, texture), instance_buffer) in instance_buffers {
            let uniforms = uniform! {
                vp: vp,
                camera_position: camera_position,
                tex: Sampler(&texture.inner_texture, sample_behaviour).0
            };

            for mesh in model.meshes.iter() {
                for primitive in mesh.primitives.iter() {
                    target
                        .draw(
                            (
                                &primitive.vertex_buffer,
                                instance_buffer.per_instance().unwrap(),
                            ),
                            &primitive.index_buffer,
                            &self.model_program,
                            &uniforms,
                            &DrawParameters {
                                depth: Depth {
                                    test: DepthTest::IfLess,
                                    write: true,
                                    ..Default::default()
                                },
                                ..DrawParameters::default()
                            },
                        )
                        .unwrap();
                }
            }
        }
    }

    fn render_lines(&mut self, display: &Display<WindowSurface>, target: &mut Frame) {
        self.line_vertex_buffers
            .get_or_insert(self.build_line_vertex_buffers(display));

        let uniforms = uniform! {
            vp: maths::raw_matrix(self.camera.view_projection),
        };

        for (width, line_points) in self.line_vertex_buffers.iter().flatten() {
            target
                .draw(
                    line_points,
                    &NoIndices(PrimitiveType::LinesList),
                    &self.lines_program,
                    &uniforms,
                    &DrawParameters {
                        line_width: Some(*width as f32),
                        ..DrawParameters::default()
                    },
                )
                .unwrap();
        }
    }

    fn build_line_vertex_buffers(
        &self,
        display: &Display<WindowSurface>,
    ) -> Vec<(u8, VertexBuffer<LinePoint>)> {
        let mut lines_map = HashMap::<u8, Vec<LinePoint>>::new();

        for line in self.lines.clone().into_iter() {
            let line_points = vec![
                LinePoint {
                    position: <[f32; 3]>::from(line.p1),
                    color: *line.color.as_ref(),
                },
                LinePoint {
                    position: <[f32; 3]>::from(line.p2),
                    color: *line.color.as_ref(),
                },
            ];

            lines_map
                .entry(line.width)
                .and_modify(|lines| lines.extend(&line_points))
                .or_insert(line_points);
        }

        lines_map
            .into_iter()
            .map(|(width, lines)| (width, VertexBuffer::new(display, &lines).unwrap()))
            .collect_vec()
    }

    fn build_instance_map(
        &self,
        display: &Display<WindowSurface>,
    ) -> HashMap<(Arc<Model>, Arc<Texture>), Vec<Instance>> {
        let mut instance_map = HashMap::<(Arc<Model>, Arc<Texture>), Vec<Instance>>::new();

        for model_instance in self.model_instances.iter() {
            let transform_matrix = Matrix4::from(model_instance.transform.clone());

            let instance = Instance {
                transform: <[[f32; 4]; 4]>::from(transform_matrix),
                transform_normal: <[[f32; 4]; 4]>::from(
                    transform_matrix.invert().unwrap().transpose(),
                ),
            };

            let entry = (
                model_instance.model.clone(),
                model_instance
                    .texture
                    .as_ref()
                    .unwrap_or(
                        &texture::load("assets/textures/uv-test.jpg".into(), display).unwrap(),
                    )
                    .clone(),
            );

            instance_map
                .entry(entry)
                .or_insert(vec![instance])
                .push(instance);
        }

        instance_map
    }

    fn build_instance_buffers(
        &self,
        display: &Display<WindowSurface>,
    ) -> Vec<((Arc<Model>, Arc<Texture>), VertexBuffer<Instance>)> {
        let instance_map = self.build_instance_map(display);

        instance_map
            .into_iter()
            .map(|((model, texture), instances)| {
                (
                    (model, texture),
                    VertexBuffer::new(display, &instances).unwrap(),
                )
            })
            .collect_vec()
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
        s.serialize_field("title", &self.title)?;

        s.end()
    }
}

struct UnloadedScene {
    pub camera: Camera,
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
            &["models_to_transforms", "camera", "title"],
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
            models_to_transforms: HashMap::new(),
        };

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "model_instances" => {
                    unloaded_scene.models_to_transforms =
                        map.next_value::<HashMap<String, HashMap<String, Vec<Transform>>>>()?
                }
                "camera" => unloaded_scene.camera = map.next_value::<Camera>()?,
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

#[derive(Copy, Clone)]
struct Instance {
    transform: [[f32; 4]; 4],
    transform_normal: [[f32; 4]; 4],
}
implement_vertex!(Instance, transform, transform_normal);
