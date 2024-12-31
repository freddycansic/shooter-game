use cgmath::{Matrix, Matrix4, Point3, SquareMatrix};
use glium::glutin::surface::WindowSurface;
use glium::{
    implement_vertex, uniform, Depth, DepthTest, Display, DrawParameters, Frame, Program, Surface,
    VertexBuffer,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::line::{Line, LinePoint};
use crate::model::Model;
use crate::model_instance::ModelInstance;
use crate::texture::Texture;
use crate::{context, maths};
use color_eyre::Result;
use glium::index::{NoIndices, PrimitiveType};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerBehavior};
use itertools::Itertools;
use petgraph::stable_graph::NodeReferences;

pub struct Renderer {
    default_program: Program,
    lines_program: Program,

    line_vertex_buffers: HashMap<u8, VertexBuffer<LinePoint>>,
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

        Ok(Self {
            default_program,
            lines_program,
            line_vertex_buffers: HashMap::new(),
        })
    }

    pub fn render_model_instances(
        &mut self,
        model_instances: NodeReferences<ModelInstance>,
        camera_view_projection: &Matrix4<f32>,
        camera_position: Point3<f32>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        let batched_instances = Self::batch_model_instances(model_instances, display);

        let vp = maths::raw_matrix(camera_view_projection.clone());
        let camera_position = <[f32; 3]>::from(camera_position);

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        for (model, texture, instance_buffer) in batched_instances {
            let uniforms = uniform! {
                vp: vp,
                camera_position: camera_position,
                tex: Sampler(texture.inner_texture.as_ref().unwrap(), sample_behaviour).0
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

    pub fn render_lines(
        &mut self,
        lines: &[Line],
        camera_view_projection: &Matrix4<f32>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        let batched_lines = Self::batch_lines(lines);

        self.write_lines_to_vertex_buffers(display, batched_lines);

        let uniforms = uniform! {
            vp: maths::raw_matrix(camera_view_projection.clone()),
        };

        for (width, line_points) in self.line_vertex_buffers.iter() {
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

    fn write_lines_to_vertex_buffers(&mut self, display: &Display<WindowSurface>, batched_lines: HashMap<u8, Vec<LinePoint>>) {
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

    /// Batches instances with the same model and texture
    fn batch_model_instances(model_instances: NodeReferences<ModelInstance>, display: &Display<WindowSurface>) -> Vec<(Arc<Model>, Arc<Texture>, VertexBuffer<Instance>)> {
        let instance_map = Self::group_instances_on_model_and_texture(model_instances, display);

        instance_map
            .into_iter()
            .map(|((model, texture), instances)| (model, texture, VertexBuffer::new(display, &instances).unwrap()))
            .collect_vec()
    }

    fn group_instances_on_model_and_texture(model_instances: NodeReferences<ModelInstance>, display: &Display<WindowSurface>) -> HashMap<(Arc<Model>, Arc<Texture>), Vec<Instance>> {
        let mut instance_map = HashMap::<(Arc<Model>, Arc<Texture>), Vec<Instance>>::new();

        for (_, model_instance) in model_instances {
            if model_instance.model.meshes.lock().unwrap().is_some() {
                let transform_matrix = Matrix4::from(model_instance.transform.clone());

                let instance = Instance {
                    transform: <[[f32; 4]; 4]>::from(transform_matrix),
                    transform_normal: <[[f32; 4]; 4]>::from(
                        transform_matrix.invert().unwrap().transpose(),
                    ),
                };

                let texture = match &model_instance.texture {
                    Some(texture) => {
                        if texture.inner_texture.is_some() {
                            texture.clone()
                        } else {
                            Texture::default(display).unwrap().clone()
                        }
                    }
                    None => Texture::default(display).unwrap().clone(),
                };

                instance_map
                    .entry((model_instance.model.clone(), texture))
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
    transform_normal: [[f32; 4]; 4],
}
implement_vertex!(Instance, transform, transform_normal);
