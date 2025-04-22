use std::hash::Hash;
use std::sync::Arc;

use color_eyre::Result;
use glium::draw_parameters::{PolygonOffset, Stencil};
use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use glium::index::{IndicesSource, NoIndices, PrimitiveType};
use glium::texture::{MipmapsOption, StencilTexture2d, Texture2d, UncompressedFloatFormat};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerBehavior};
use glium::vertex::EmptyVertexAttributes;
use glium::{
    BackfaceCullingMode, Blend, BlitMask, BlitTarget, Depth, DepthTest, Display, DrawParameters,
    Frame, PolygonMode, Program, Rect, StencilOperation, StencilTest, Surface, Vertex,
    VertexBuffer, implement_vertex, uniform,
};
use gxhash::{HashMap, HashMapExt};
use itertools::Itertools;
use petgraph::prelude::StableDiGraph;
use rapier3d::na::{Matrix4, Point3};
use uuid::Uuid;

use crate::colors::{self, ColorExt};
use crate::light::{Light, ShaderLight};
use crate::line::{Line, LinePoint};
use crate::maths::Matrix4Ext;
use crate::models::primitives::SimplePoint;
use crate::models::{Material, ModelInstance};
use crate::models::{Model, primitives};
use crate::quad::{Quad, QuadVertex};
use crate::terrain::Terrain;
use crate::texture::Cubemap;
use crate::{context, maths};

pub struct Renderer {
    orthograhic_projection: Matrix4<f32>,
    perspective_projection: Matrix4<f32>,

    outline_program: Program,

    default_program: Program,
    model_instance_buffers: HashMap<(Arc<Model>, Material, bool), VertexBuffer<Instance>>,

    skybox_program: Program,
    light_program: Program,
    cube_vertex_buffer: VertexBuffer<SimplePoint>,

    lines_program: Program,
    line_vertex_buffers: HashMap<u8, VertexBuffer<LinePoint>>,

    terrain_program: Program,

    quad_program: Program,
    quad_vertex_buffers: HashMap<Uuid, VertexBuffer<QuadVertex>>,

    fullscreen_quad_program: Program,
    solid_color_program: Program,
}

impl Renderer {
    pub fn new(
        window_width: f32,
        window_height: f32,
        display: &Display<WindowSurface>,
    ) -> Result<Self> {
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

        let quad_program = context::new_program(
            "assets/shaders/quad/quad.vert",
            "assets/shaders/quad/quad.frag",
            Some("assets/shaders/quad/quad.geom"),
            display,
        )?;

        let outline_program = context::new_program(
            "assets/shaders/outline/outline.vert",
            "assets/shaders/outline/outline.frag",
            None,
            display,
        )?;

        let fullscreen_quad_program = context::new_program(
            "assets/shaders/fullscreen_quad/fullscreen_quad.vert",
            "assets/shaders/fullscreen_quad/fullscreen_quad.frag",
            None,
            display,
        )?;

        let solid_color_program = context::new_program(
            "assets/shaders/solid_color/solid_color.vert",
            "assets/shaders/solid_color/solid_color.frag",
            None,
            display,
        )?;

        // This will be used by the skybox and debug lights
        let cube_vertex_buffer = VertexBuffer::new(display, &primitives::CUBE)?;

        Ok(Self {
            perspective_projection: maths::perspective_matrix_from_window_size(
                window_width,
                window_height,
            ),
            orthograhic_projection: maths::orthographic_matrix_from_window_size(
                window_width,
                window_height,
            ),
            outline_program,
            default_program,
            model_instance_buffers: HashMap::new(),
            skybox_program,
            light_program,
            cube_vertex_buffer,
            lines_program,
            line_vertex_buffers: HashMap::new(),
            terrain_program,
            quad_program,
            quad_vertex_buffers: HashMap::new(),
            fullscreen_quad_program,
            solid_color_program,
        })
    }

    pub fn update_projection_matrices(&mut self, window_width: f32, window_height: f32) {
        self.perspective_projection =
            maths::perspective_matrix_from_window_size(window_width, window_height);

        self.orthograhic_projection =
            maths::orthographic_matrix_from_window_size(window_width, window_height);
    }

    pub fn render_model_instances(
        &mut self,
        model_instances: &StableDiGraph<ModelInstance, ()>,
        view: &Matrix4<f32>,
        camera_position: Point3<f32>,
        lights: &[Light],
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        let default_material = Material::default(display).unwrap();

        let batched_instances = model_instances
            .node_weights()
            .into_group_map_by(|model_instance| {
                let model = model_instance.model.clone();
                let material = match &model_instance.material {
                    Some(material) => material.clone(),
                    None => default_material.clone(),
                };
                (model, material, model_instance.selected)
            })
            .into_iter()
            .map(|(key, model_instances)| {
                (
                    key,
                    model_instances
                        .iter()
                        .map(|model_instance| Instance {
                            transform: model_instance.transform.matrix(),
                        })
                        .collect_vec(),
                )
            })
            .collect_vec();

        let vp = maths::raw_matrix(self.perspective_projection * view);
        let camera_position = <[f32; 3]>::from(camera_position);

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let solid_color_uniforms = uniform! {
            vp: vp,
        };

        let dimensions = target.get_dimensions();
        let mask_texture = Texture2d::empty_with_format(
            display,
            UncompressedFloatFormat::U8,
            MipmapsOption::NoMipmap,
            dimensions.0,
            dimensions.1,
        )
        .unwrap();
        let mut framebuffer = SimpleFrameBuffer::new(display, &mask_texture).unwrap();

        // Only draw selected models into mask
        for ((model, material, selected), instances) in batched_instances
            .iter()
            .filter(|((_, _, selected), _)| *selected)
        {
            Self::copy_into_buffers(
                display,
                (model.clone(), material.clone(), true),
                &instances,
                &mut self.model_instance_buffers,
            );

            for mesh in model.clone().meshes.lock().unwrap().iter().flatten() {
                for primitive in mesh.primitives.iter() {
                    framebuffer
                        .draw(
                            (
                                &primitive.vertex_buffer,
                                self.model_instance_buffers
                                    .get(&(model.clone(), material.clone(), true))
                                    .unwrap()
                                    .slice(0..instances.len())
                                    .unwrap()
                                    .per_instance()
                                    .unwrap(),
                            ),
                            &primitive.index_buffer,
                            &self.solid_color_program,
                            &solid_color_uniforms,
                            &DrawParameters::default(),
                        )
                        .unwrap();
                }
            }
        }

        // Draw regular color buffer
        for ((model, material, selected), instances) in batched_instances.iter() {
            Self::copy_into_buffers(
                display,
                (model.clone(), material.clone(), *selected),
                &instances,
                &mut self.model_instance_buffers,
            );

            let uniforms = uniform! {
                vp: vp,
                camera_position: camera_position,
                // TODO temporary
                light_color: <[f32; 3]>::from(lights.iter().next().unwrap_or(&Light::default()).color.to_rgb_vector3()),
                light_position: <[f32; 3]>::from(lights.iter().next().unwrap_or(&Light::default()).position),
                diffuse_texture: Sampler(material.diffuse.inner_texture.as_ref().unwrap(), sample_behaviour).0,
                specular_texture: Sampler(material.specular.inner_texture.as_ref().unwrap(), sample_behaviour).0,
            };

            for mesh in model.clone().meshes.lock().unwrap().iter().flatten() {
                for primitive in mesh.primitives.iter() {
                    target
                        .draw(
                            (
                                &primitive.vertex_buffer,
                                self.model_instance_buffers
                                    .get(&(model.clone(), material.clone(), *selected))
                                    .unwrap()
                                    .slice(0..instances.len())
                                    .unwrap()
                                    .per_instance()
                                    .unwrap(),
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
                                backface_culling: BackfaceCullingMode::CullClockwise,
                                ..DrawParameters::default()
                            },
                        )
                        .unwrap();
                }
            }
        }

        // Dilate selection mask
        let dilated_texture = Texture2d::empty_with_format(
            display,
            UncompressedFloatFormat::U8U8U8U8,
            MipmapsOption::NoMipmap,
            dimensions.0,
            dimensions.1,
        )
        .unwrap();
        let mut dilate_framebuffer = SimpleFrameBuffer::new(display, &dilated_texture).unwrap();

        let dilate_uniforms = uniform! {
            mask_texture: mask_texture,
            outline_color: <[f32; 3]>::from(colors::SELECTED.to_rgb_vector3()),
            outline_radius: 2
        };

        dilate_framebuffer
            .draw(
                EmptyVertexAttributes { len: 4 },
                IndicesSource::NoIndices {
                    primitives: PrimitiveType::TriangleStrip,
                },
                &self.outline_program,
                &dilate_uniforms,
                &DrawParameters::default(),
            )
            .unwrap();

        let fullscreen_quad_uniforms = uniform! {
            fullscreen_quad_texture: dilated_texture,
        };

        // Draw outline
        target
            .draw(
                EmptyVertexAttributes { len: 4 },
                IndicesSource::NoIndices {
                    primitives: PrimitiveType::TriangleStrip,
                },
                &self.fullscreen_quad_program,
                &fullscreen_quad_uniforms,
                &DrawParameters {
                    depth: Depth {
                        test: DepthTest::Overwrite,
                        write: false,
                        ..Default::default()
                    },
                    blend: Blend::alpha_blending(),
                    ..DrawParameters::default()
                },
            )
            .unwrap();
    }

    pub fn render_terrain(
        &mut self,
        terrain: &Terrain,
        view: &Matrix4<f32>,
        camera_position: Point3<f32>,
        target: &mut Frame,
    ) {
        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let uniforms = uniform! {
            vp: maths::raw_matrix(self.perspective_projection * view),
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

    pub fn render_quads(
        &mut self,
        quads: &StableDiGraph<Quad, ()>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        if quads.node_count() == 0 {
            return;
        }

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let grouped_quad_vertices = quads
            .node_weights()
            .cloned()
            .into_group_map_by(|quad| quad.texture.clone())
            .into_iter()
            .map(|(texture, quads)| {
                (
                    texture,
                    quads.into_iter().map(QuadVertex::from).collect_vec(),
                )
            });

        for (texture, quad_vertices) in grouped_quad_vertices {
            Self::copy_into_buffers(
                display,
                texture.uuid,
                &quad_vertices,
                &mut self.quad_vertex_buffers,
            );

            let uniforms = uniform! {
                diffuse_texture: Sampler(texture.inner_texture.as_ref().unwrap(), sample_behaviour),
                projection: maths::raw_matrix(self.orthograhic_projection.clone())
            };

            target
                .draw(
                    self.quad_vertex_buffers
                        .get(&texture.uuid)
                        .unwrap()
                        .slice(0..quad_vertices.len())
                        .unwrap(),
                    NoIndices(PrimitiveType::Points),
                    &self.quad_program,
                    &uniforms,
                    &DrawParameters {
                        // Depth buffer is disabled so that they appear on top
                        blend: Blend::alpha_blending(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }

    pub fn render_skybox(&mut self, cubemap: &Cubemap, view: &Matrix4<f32>, target: &mut Frame) {
        // Strip translation from view matrix = skybox is always in the same place
        let stripped_view = view.stripped_w();
        let vp = self.perspective_projection * stripped_view;

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let uniforms = uniform! {
            vp: maths::raw_matrix(vp),
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
        view: &Matrix4<f32>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        if lines.is_empty() {
            return;
        }

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

        let uniforms = uniform! {
            vp: maths::raw_matrix(self.perspective_projection * view),
        };

        for (width, line_points) in batched_lines {
            Self::copy_into_buffers(display, width, &line_points, &mut self.line_vertex_buffers);

            target
                .draw(
                    self.line_vertex_buffers
                        .get(&width)
                        .unwrap()
                        .slice(0..line_points.len())
                        .unwrap(),
                    NoIndices(PrimitiveType::LinesList),
                    &self.lines_program,
                    &uniforms,
                    &DrawParameters {
                        line_width: Some(width as f32),
                        ..DrawParameters::default()
                    },
                )
                .unwrap();
        }
    }

    pub fn render_lights(
        &mut self,
        lights: &[Light],
        view: &Matrix4<f32>,
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
            vp: maths::raw_matrix(self.perspective_projection * view),
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

    fn copy_into_buffers<T, V>(
        display: &Display<WindowSurface>,
        key: T,
        vertices: &Vec<V>,
        vertex_buffers: &mut HashMap<T, VertexBuffer<V>>,
    ) where
        T: Hash + Eq,
        V: Copy + Vertex,
    {
        if let Some(quad_vertex_buffer) = vertex_buffers.get_mut(&key) {
            // If the allocated vertex buffer is too small, then double the size
            if quad_vertex_buffer.len() < vertices.len() {
                *quad_vertex_buffer = context::new_sized_dynamic_vertex_buffer_with_data(
                    display,
                    quad_vertex_buffer.len() * 2,
                    vertices,
                )
                .unwrap();
            // If it is big enough then write quads
            } else {
                quad_vertex_buffer
                    .slice_mut(..vertices.len())
                    .unwrap()
                    .write(vertices);
            }
        // If the vertex buffer does not exist then make one at least as big as INITIAL_VERTEX_BUFFER_SIZE
        } else {
            // Smallest 2^x above current len
            // Example
            // quad_vertices.len() = 150
            // 100.log2() = ~7.23
            // 7.23.ceil() = 8
            // 2.pow(8) = 256
            // min_size = 256.max(INITIAL_VERTEX_BUFFER_SIZE [128]) = 256
            let x = (vertices.len() as f64).log2().ceil() as u32;
            const INITIAL_VERTEX_BUFFER_SIZE: u32 = 128;
            let min_size = 2_u32.pow(x).max(INITIAL_VERTEX_BUFFER_SIZE);

            vertex_buffers.insert(
                key,
                context::new_sized_dynamic_vertex_buffer_with_data(
                    display,
                    min_size as usize,
                    vertices,
                )
                .unwrap(),
            );
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Instance {
    transform: [[f32; 4]; 4],
}
implement_vertex!(Instance, transform);
