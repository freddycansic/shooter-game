use cgmath::{Matrix, Matrix4, SquareMatrix};
use glium::glutin::surface::WindowSurface;
use glium::{
    implement_vertex, uniform, Depth, DepthTest, Display, DrawParameters, Frame, Program, Surface,
    VertexBuffer,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::camera::Camera;
use crate::line::{Line, LinePoint};
use crate::model::{Model, ModelInstance};
use crate::texture::Texture;
use crate::{context, maths, texture};
use color_eyre::Result;
use glium::index::{NoIndices, PrimitiveType};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerBehavior};
use itertools::Itertools;

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
        model_instances: &[ModelInstance],
        camera: &Camera,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        let mut instance_map = HashMap::<(Arc<Model>, Arc<Texture>), Vec<Instance>>::new();

        for model_instance in model_instances.iter() {
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

        let instance_buffers = instance_map
            .into_iter()
            .map(|((model, texture), instances)| {
                (
                    (model, texture),
                    VertexBuffer::new(display, &instances).unwrap(),
                )
            })
            .collect_vec();

        let vp = maths::raw_matrix(camera.view_projection);
        let camera_position = <[f32; 3]>::from(camera.position);

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
        camera: &Camera,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
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

        for (width, lines) in batched_lines.iter() {
            if self.line_vertex_buffers.contains_key(width) {
                self.line_vertex_buffers.get(width).unwrap().write(lines);
            } else {
                self.line_vertex_buffers
                    .insert(*width, VertexBuffer::dynamic(display, lines).unwrap());
            }
        }

        let uniforms = uniform! {
            vp: maths::raw_matrix(camera.view_projection),
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
}

#[derive(Copy, Clone)]
struct Instance {
    transform: [[f32; 4]; 4],
    transform_normal: [[f32; 4]; 4],
}
implement_vertex!(Instance, transform, transform_normal);
