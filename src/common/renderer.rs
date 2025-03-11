use crate::colors::ColorExt;
use crate::light::{Light, ShaderLight};
use crate::line::{Line, LinePoint};
use crate::models::primitives::SimplePoint;
use crate::models::{primitives, Model};
use crate::models::{Material, ModelInstance};
use crate::terrain::Terrain;
use crate::texture::Cubemap;
use crate::{context, maths};
use cgmath::{Matrix3, Matrix4, Point3};
use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::index::{NoIndices, PrimitiveType};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerBehavior};
use glium::{
    implement_vertex, uniform, Depth, DepthTest, Display, DrawParameters, Frame, Program, Surface,
    VertexBuffer,
};
use itertools::Itertools;
use petgraph::stable_graph::NodeReferences;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Renderer {
    default_program: Program,

    skybox_program: Program,
    light_program: Program,
    cube_vertex_buffer: VertexBuffer<SimplePoint>,

    lines_program: Program,
    line_vertex_buffers: HashMap<u8, VertexBuffer<LinePoint>>,

    terrain_program: Program,
}

impl Renderer {
    pub fn new(display: &Display<WindowSurface>) -> Result<Self> {
        let default_program = context::new_program(
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

        let skybox_program = context::new_program(
            "assets/shaders/skybox/skybox.vert",
            "assets/shaders/skybox/skybox.frag",
            None,
            display,
        )?;

        let light_program = context::new_program(
            "assets/shaders/light/light.vert",
            "assets/shaders/light/light.frag",
            None,
            display,
        )?;

        let terrain_program = context::new_program(
            "assets/shaders/terrain/terrain.vert",
            "assets/shaders/terrain/terrain.frag",
            None,
            display,
        )?;

        // This will be used by the skybox and debug lights
        let cube_vertex_buffer = VertexBuffer::new(display, &primitives::CUBE)?;

        Ok(Self {
            default_program,
            skybox_program,
            light_program,
            cube_vertex_buffer,
            lines_program,
            line_vertex_buffers: HashMap::new(),
            terrain_program,
        })
    }

    pub fn render_model_instances(
        &mut self,
        model_instances: NodeReferences<ModelInstance>,
        camera_view_projection: &Matrix4<f32>,
        camera_position: Point3<f32>,
        lights: &[Light],
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        let batched_instances = Self::batch_model_instances(model_instances, display);

        let vp = maths::raw_matrix(*camera_view_projection);
        let camera_position = <[f32; 3]>::from(camera_position);

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        for (model, material, instance_buffer) in batched_instances {
            let uniforms = uniform! {
                vp: vp,
                camera_position: camera_position,
                // TODO temporary
                light_color: <[f32; 3]>::from(lights.iter().next().unwrap_or(&Light::default()).color.to_rgb_vector3()),
                light_position: <[f32; 3]>::from(lights.iter().next().unwrap_or(&Light::default()).position),
                diffuse_texture: Sampler(material.diffuse.inner_texture.as_ref().unwrap(), sample_behaviour).0,
                specular_texture: Sampler(material.specular.inner_texture.as_ref().unwrap(), sample_behaviour).0,
            };

            for mesh in model.meshes.lock().unwrap().iter().flatten() {
                for primitive in mesh.primitives.iter() {
                    target
                        .draw(
                            (
                                &primitive.vertex_buffer,
                                instance_buffer.per_instance().unwrap(),
                            ),
                            &primitive.index_buffer,
                            &self.default_program,
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

    pub fn render_terrain(
        &mut self,
        terrain: &Terrain,
        view_projection: &Matrix4<f32>,
        camera_position: Point3<f32>,
        target: &mut Frame,
    ) {
        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let uniforms = uniform! {
            vp: maths::raw_matrix(*view_projection),
            camera_position: <[f32; 3]>::from(camera_position),
            diffuse_texture: Sampler(terrain.material.diffuse.inner_texture.as_ref().unwrap(), sample_behaviour).0
        };

        target
            .draw(
                terrain.vertex_buffer.as_ref().unwrap(),
                NoIndices(PrimitiveType::TrianglesList),
                &self.terrain_program,
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
            .unwrap()
    }

    pub fn render_skybox(
        &mut self,
        cubemap: &Cubemap,
        view: &Matrix4<f32>,
        projection: &Matrix4<f32>,
        target: &mut Frame,
    ) {
        // Strip translation from view matrix = skybox is always in the same place
        let view = Matrix4::from(Matrix3::from_cols(view.x.xyz(), view.y.xyz(), view.z.xyz()));
        let view_projection = projection * view;

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let uniforms = uniform! {
            vp: maths::raw_matrix(view_projection),
            skybox: Sampler(cubemap.inner_cubemap.as_ref().unwrap(), sample_behaviour).0
        };

        target
            .draw(
                &self.cube_vertex_buffer,
                NoIndices(PrimitiveType::TrianglesList),
                &self.skybox_program,
                &uniforms,
                &DrawParameters::default(),
            )
            .unwrap();
    }

    pub fn render_lines(
        &mut self,
        lines: &[Line],
        camera_view_projection: &Matrix4<f32>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        if lines.is_empty() {
            return;
        }

        let batched_lines = Self::batch_lines(lines);

        self.write_lines_to_vertex_buffers(display, batched_lines);

        let uniforms = uniform! {
            vp: maths::raw_matrix(*camera_view_projection),
        };

        for (width, line_points) in self.line_vertex_buffers.iter() {
            target
                .draw(
                    line_points,
                    NoIndices(PrimitiveType::LinesList),
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

    pub fn render_lights(
        &mut self,
        lights: &[Light],
        camera_view_projection: &Matrix4<f32>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        if lights.is_empty() {
            return;
        }

        let shader_lights = lights
            .iter()
            .map(|light| ShaderLight::from(light.clone()))
            .collect_vec();

        let light_instance_buffer = VertexBuffer::new(display, &shader_lights).unwrap();

        let uniforms = uniform! {
            vp: maths::raw_matrix(*camera_view_projection),
        };

        target
            .draw(
                (
                    &self.cube_vertex_buffer,
                    light_instance_buffer.per_instance().unwrap(),
                ),
                NoIndices(PrimitiveType::TrianglesList),
                &self.light_program,
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

    fn write_lines_to_vertex_buffers(
        &mut self,
        display: &Display<WindowSurface>,
        batched_lines: HashMap<u8, Vec<LinePoint>>,
    ) {
        for (width, lines) in batched_lines.iter() {
            if self.line_vertex_buffers.contains_key(width) {
                self.line_vertex_buffers.get(width).unwrap().write(lines);
            } else {
                self.line_vertex_buffers
                    .insert(*width, VertexBuffer::dynamic(display, lines).unwrap());
            }
        }
    }

    fn batch_lines(lines: &[Line]) -> HashMap<u8, Vec<LinePoint>> {
        let mut batched_lines = HashMap::<u8, Vec<LinePoint>>::new();

        for line in lines.iter() {
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

            batched_lines
                .entry(line.width)
                .and_modify(|lines| lines.extend(&line_points))
                .or_insert(line_points);
        }
        batched_lines
    }

    /// Batches instances with the same models and texture
    #[allow(clippy::mutable_key_type)]
    fn batch_model_instances(
        model_instances: NodeReferences<ModelInstance>,
        display: &Display<WindowSurface>,
    ) -> Vec<(Arc<Model>, Material, VertexBuffer<Instance>)> {
        let instance_map = Self::group_instances_on_model_and_texture(model_instances, display);

        instance_map
            .into_iter()
            .map(|((model, texture), instances)| {
                (
                    model,
                    texture,
                    // TODO cache vertex buffers and write over them on next frame
                    VertexBuffer::new(display, &instances).unwrap(),
                )
            })
            .collect_vec()
    }

    #[allow(clippy::mutable_key_type)]
    fn group_instances_on_model_and_texture(
        model_instances: NodeReferences<ModelInstance>,
        display: &Display<WindowSurface>,
    ) -> HashMap<(Arc<Model>, Material), Vec<Instance>> {
        let mut instance_map = HashMap::<(Arc<Model>, Material), Vec<Instance>>::new();

        for (_, model_instance) in model_instances {
            if model_instance.model.meshes.lock().unwrap().is_some() {
                let transform_matrix = Matrix4::from(model_instance.transform.clone());

                let instance = Instance {
                    transform: maths::raw_matrix(transform_matrix),
                };

                let material = match &model_instance.material {
                    Some(material) => material.clone(),
                    None => Material::default(display).unwrap().clone(),
                };

                instance_map
                    .entry((model_instance.model.clone(), material))
                    .or_insert(vec![instance])
                    .push(instance);
            }
        }
        instance_map
    }
}

#[derive(Copy, Clone)]
struct Instance {
    transform: [[f32; 4]; 4],
}
implement_vertex!(Instance, transform);
