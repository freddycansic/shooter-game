use std::collections::hash_map::Entry;
use std::hash::Hash;

use color_eyre::Result;
use egui_glium::egui_winit::egui::{self, Pos2};
use fxhash::{FxBuildHasher, FxHashMap};
use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use glium::index::{IndicesSource, NoIndices, PrimitiveType};
use glium::texture::{MipmapsOption, Texture2d, UncompressedFloatFormat};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerBehavior};
use glium::vertex::EmptyVertexAttributes;
use glium::{Blend, Depth, DepthTest, Display, DrawParameters, Frame, Program, Surface, Vertex, VertexBuffer, implement_vertex, uniform, BlendingFunction, LinearBlendingFactor};
use itertools::Itertools;
use nalgebra::{Matrix4, Point3, Translation3};

use crate::colors::{self, ColorExt};
use crate::debug::DebugCuboid;
use crate::geometry::primitives;
use crate::geometry::primitives::SimplePoint;
use crate::input::Input;
use crate::light::Light;
use crate::line::{Line, LinePoint};
use crate::maths::Matrix4Ext;
use crate::quad::QuadVertex;
use crate::resources::Resources;
use crate::resources::{CubemapHandle, TextureHandle};
use crate::scene::QuadBatches;
use crate::scene::graph::{GeometryBatchKey, GeometryBatches};
use crate::scene::scene::RenderQueue;
use crate::{context, maths};

struct Programs {
    outline: Program,
    default: Program,
    skybox: Program,
    light: Program,
    lines: Program,
    terrain: Program,
    quad: Program,
    fullscreen_quad: Program,
    solid_color: Program,
    white: Program,
}

pub struct RendererBuffers {
    pub instance_buffers: FxHashMap<GeometryBatchKey, VertexBuffer<Instance>>,
    pub line_vertex_buffers: FxHashMap<u8, VertexBuffer<LinePoint>>,
    pub quad_vertex_buffers: FxHashMap<TextureHandle, VertexBuffer<QuadVertex>>,
    pub cube_vertex_buffer: VertexBuffer<SimplePoint>,
    pub debug_cube_instance_buffer: VertexBuffer<SolidColorInstance>,
}

impl RendererBuffers {
    pub fn get_vertex_buffer<'a, K, V>(
        buffers: &'a mut FxHashMap<K, VertexBuffer<V>>,
        key: &K,
        data: &[V],
        display: &Display<WindowSurface>,
    ) -> &'a VertexBuffer<V>
    where
        K: Eq + Hash + Clone,
        V: Vertex,
    {
        match buffers.entry(key.clone()) {
            Entry::Occupied(entry) => {
                let buffer = entry.into_mut();
                Self::ensure_vertex_buffer_size(buffer, data, display);
                buffer
            }
            // If the vertex buffer does not exist then make one at least as big as INITIAL_VERTEX_BUFFER_SIZE

            // Smallest 2^x above current len
            // Example
            // quad_vertices.len() = 150
            // 100.log2() = ~7.23
            // 7.23.ceil() = 8
            // 2.pow(8) = 256
            // min_size = 256.max(INITIAL_VERTEX_BUFFER_SIZE [128]) = 256
            Entry::Vacant(entry) => {
                const INITIAL_VERTEX_BUFFER_SIZE: usize = 128;
                let x = (data.len() as f64).log2().ceil() as u32;
                let min_size = 2_u32.pow(x).max(INITIAL_VERTEX_BUFFER_SIZE as u32) as usize;

                let buffer = context::new_sized_dynamic_vertex_buffer_with_data(display, min_size, data).unwrap();
                entry.insert(buffer)
            }
        }
    }

    pub fn ensure_vertex_buffer_size<V>(buffer: &mut VertexBuffer<V>, data: &[V], display: &Display<WindowSurface>)
    where
        V: Vertex,
    {
        if buffer.len() < data.len() {
            // double the size of existing buffer or fit data
            let new_size = (buffer.len() * 2).max(data.len());
            *buffer = context::new_sized_dynamic_vertex_buffer_with_data(display, new_size, data).unwrap();
        } else {
            buffer.slice_mut(..data.len()).unwrap().write(data);
        }
    }
}

pub struct Renderer {
    orthograhic_projection: Matrix4<f32>,
    perspective_projection: Matrix4<f32>,

    buffers: RendererBuffers,
    programs: Programs,

    pub viewport: Option<egui::Rect>,
}

impl Renderer {
    pub fn new(
        window_width: f32,
        window_height: f32,
        viewport: Option<egui::Rect>,
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

        let white_program = context::new_program(
            "assets/shaders/white/white.vert",
            "assets/shaders/white/white.frag",
            None,
            display,
        )?;

        // This will be used by the skybox and debug lights
        let cube_vertex_buffer = VertexBuffer::new(display, &primitives::CUBE)?;

        let hasher = FxBuildHasher::new();

        Ok(Self {
            perspective_projection: maths::perspective_matrix_from_dimensions(window_width, window_height),
            orthograhic_projection: maths::orthographic_matrix_from_dimensions(window_width, window_height),
            buffers: RendererBuffers {
                instance_buffers: FxHashMap::with_hasher(hasher.clone()),
                line_vertex_buffers: FxHashMap::with_hasher(hasher.clone()),
                quad_vertex_buffers: FxHashMap::with_hasher(hasher),
                debug_cube_instance_buffer: VertexBuffer::empty(display, 10 /* ? */).unwrap(),
                cube_vertex_buffer,
            },
            programs: Programs {
                outline: outline_program,
                default: default_program,
                skybox: skybox_program,
                light: light_program,
                lines: lines_program,
                terrain: terrain_program,
                quad: quad_program,
                fullscreen_quad: fullscreen_quad_program,
                solid_color: solid_color_program,
                white: white_program,
            },
            viewport,
        })
    }

    pub fn perspective_projection(&self) -> Matrix4<f32> {
        self.perspective_projection
    }

    pub fn orthographic_projection(&self) -> Matrix4<f32> {
        self.orthograhic_projection
    }

    pub fn update_projection_matrices(&mut self, width: f32, height: f32) {
        self.perspective_projection = maths::perspective_matrix_from_dimensions(width, height);
        self.orthograhic_projection = maths::orthographic_matrix_from_dimensions(width, height);
    }

    pub fn update_viewport(&mut self, viewport: egui::Rect) {
        self.update_projection_matrices(viewport.width(), viewport.height());
        self.viewport = Some(viewport);
    }

    pub fn is_mouse_in_viewport(&self, input: &Input) -> bool {
        if !input.mouse_on_window() {
            return false;
        }

        input.mouse_position().is_some_and(|position| {
            self.viewport
                .is_some_and(|viewport| viewport.contains(Pos2::new(position.x as f32, position.y as f32)))
        })
    }

    fn get_glium_viewport(&self) -> Option<glium::Rect> {
        // Convert to bottom left rect from top right
        self.viewport.map(|viewport| glium::Rect {
            left: viewport.min.x as u32,
            bottom: (viewport.height() - viewport.max.y) as u32,
            width: viewport.width() as u32,
            height: viewport.height() as u32,
        })
    }

    pub fn render_queue(
        &mut self,
        queue: RenderQueue,
        resources: &Resources,
        view: &Matrix4<f32>,
        camera_position: Point3<f32>,
        lights: &[Light],
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        let vp = maths::raw_matrix(self.perspective_projection * view);

        let dimensions = target.get_dimensions();

        let mask_texture = self.render_mask_texture(&queue.geometry_batches, resources, dimensions, &vp, display);

        self.render_model_instances_color(
            &queue.geometry_batches,
            resources,
            &vp,
            lights,
            camera_position,
            display,
            target,
        );

        let outline_texture = self.render_outline_texture(mask_texture, dimensions, display);
        self.render_outline(outline_texture, target);

        // Render quads last so they stay on top
        self.render_quads(&queue.quad_batches, resources, display, target);
    }

    // pub fn render_terrain(
    //     &mut self,
    //     terrain: &Terrain,
    //     view: &Matrix4<f32>,
    //     camera_position: Point3<f32>,
    //     target: &mut Frame,
    // ) {
    //     let sample_behaviour = SamplerBehavior {
    //         minify_filter: MinifySamplerFilter::Nearest,
    //         magnify_filter: MagnifySamplerFilter::Nearest,
    //         ..SamplerBehavior::default()
    //     };

    //     let uniforms = uniform! {
    //         vp: maths::raw_matrix(self.perspective_projection * view),
    //         camera_position: <[f32; 3]>::from(camera_position),
    //         diffuse_texture: Sampler(terrain.material.diffuse.inner_texture.as_ref().unwrap(), sample_behaviour).0
    //     };

    //     target
    //         .draw(
    //             terrain.vertex_buffer.as_ref().unwrap(),
    //             NoIndices(PrimitiveType::TrianglesList),
    //             &self.terrain_program,
    //             &uniforms,
    //             &DrawParameters {
    //                 depth: Depth {
    //                     test: DepthTest::IfLess,
    //                     write: true,
    //                     ..Default::default()
    //                 },
    //                 ..DrawParameters::default()
    //             },
    //         )
    //         .unwrap()
    // }

    pub fn render_quads(
        &mut self,
        quad_batches: &QuadBatches,
        resources: &Resources,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let viewport = self.get_glium_viewport();

        for (texture_handle, quad_vertices) in quad_batches.iter() {
            let quad_buffer = RendererBuffers::get_vertex_buffer(
                &mut self.buffers.quad_vertex_buffers,
                &texture_handle,
                quad_vertices,
                display,
            );

            let texture = resources.get_texture(*texture_handle);

            let uniforms = uniform! {
                diffuse_texture: Sampler(&texture.inner_texture, sample_behaviour),
                projection: maths::raw_matrix(self.orthograhic_projection)
            };

            target
                .draw(
                    quad_buffer.slice(0..quad_vertices.len()).unwrap(),
                    NoIndices(PrimitiveType::Points),
                    &self.programs.quad,
                    &uniforms,
                    &DrawParameters {
                        // Depth buffer is disabled so that they appear on top
                        blend: Blend::alpha_blending(),
                        viewport,
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }

    pub fn render_debug_cuboids(
        &mut self,
        cuboids: &[DebugCuboid],
        opacity: f32,
        view: &Matrix4<f32>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        assert!(opacity > 0.0 && opacity <= 1.0);

        if cuboids.is_empty() {
            return;
        }

        let instances = cuboids
            .iter()
            .map(|cuboid| {
                let scale_vec = cuboid.max - cuboid.min;
                let scaling = Matrix4::new_nonuniform_scaling(&scale_vec);

                let center = (cuboid.min + cuboid.max) * 0.5;
                let translation = Translation3::from(center);

                let transform = translation.to_homogeneous() * scaling;

                SolidColorInstance {
                    transform: transform.into(),
                    color: cuboid.color.to_rgb_vector3().try_into().unwrap(),
                }
            })
            .collect_vec();

        RendererBuffers::ensure_vertex_buffer_size(&mut self.buffers.debug_cube_instance_buffer, &instances, display);

        let vp = maths::raw_matrix(self.perspective_projection * view);

        let premultiplied_alpha = Blend {
            color: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::OneMinusSourceAlpha,
            },
            alpha: BlendingFunction::Addition {
                source: LinearBlendingFactor::One,
                destination: LinearBlendingFactor::OneMinusSourceAlpha,
            },
            constant_value: (0.0, 0.0, 0.0, 0.0),
        };

        target
            .draw(
                (
                    &self.buffers.cube_vertex_buffer,
                    self.buffers
                        .debug_cube_instance_buffer
                        .slice(0..cuboids.len())
                        .unwrap()
                        .per_instance()
                        .unwrap(),
                ),
                NoIndices(PrimitiveType::TrianglesList),
                &self.programs.solid_color,
                &uniform! {
                    vp: vp,
                    opacity: opacity,
                },
                &DrawParameters {
                    depth: Depth {
                        test: DepthTest::IfLess,
                        write: false,
                        ..Default::default()
                    },
                    blend: premultiplied_alpha,
                    viewport: self.get_glium_viewport(),
                    ..DrawParameters::default()
                },
            )
            .unwrap();

        target
            .draw(
                (
                    &self.buffers.cube_vertex_buffer,
                    self.buffers
                        .debug_cube_instance_buffer
                        .slice(0..cuboids.len())
                        .unwrap()
                        .per_instance()
                        .unwrap(),
                ),
                NoIndices(PrimitiveType::TrianglesList),
                &self.programs.solid_color,
                &uniform! {
                    vp: vp,
                    opacity: (opacity + 0.2).min(1.0),
                },
                &DrawParameters {
                    depth: Depth {
                        test: DepthTest::IfLess,
                        write: false,
                        ..Default::default()
                    },
                    blend: premultiplied_alpha,
                    polygon_mode: glium::PolygonMode::Line,
                    line_width: Some(2.0),
                    viewport: self.get_glium_viewport(),
                    ..DrawParameters::default()
                },
            )
            .unwrap();
    }

    pub fn render_skybox(
        &mut self,
        cubemap_handle: CubemapHandle,
        resources: &Resources,
        view: &Matrix4<f32>,
        target: &mut Frame,
    ) {
        // Strip translation from view matrix = skybox is always in the same place
        let stripped_view = view.stripped_w();
        let vp = self.perspective_projection * stripped_view;

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let cubemap = resources.get_cubemap(cubemap_handle);

        let uniforms = uniform! {
            vp: maths::raw_matrix(vp),
            skybox: Sampler(&cubemap.inner_cubemap, sample_behaviour).0
        };

        target
            .draw(
                &self.buffers.cube_vertex_buffer,
                NoIndices(PrimitiveType::TrianglesList),
                &self.programs.skybox,
                &uniforms,
                &DrawParameters {
                    viewport: self.get_glium_viewport(),
                    ..DrawParameters::default()
                },
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

        let mut batched = FxHashMap::<u8, Vec<LinePoint>>::with_hasher(FxBuildHasher::new());

        for line in lines {
            batched.entry(line.width).or_default().extend_from_slice(&[
                LinePoint {
                    position: line.p1.into(),
                    color: *line.color.as_ref(),
                },
                LinePoint {
                    position: line.p2.into(),
                    color: *line.color.as_ref(),
                },
            ]);
        }

        let vp = maths::raw_matrix(self.perspective_projection * view);
        let viewport = self.get_glium_viewport();

        for (width, points) in batched {
            let buffer =
                RendererBuffers::get_vertex_buffer(&mut self.buffers.line_vertex_buffers, &width, &points, display);

            target
                .draw(
                    buffer.slice(0..points.len()).unwrap(),
                    NoIndices(PrimitiveType::LinesList),
                    &self.programs.lines,
                    &uniform! { vp: vp },
                    &DrawParameters {
                        line_width: Some(width as f32),
                        viewport,
                        ..DrawParameters::default()
                    },
                )
                .unwrap();
        }
    }

    // pub fn render_lights(
    //     &mut self,
    //     lights: &[Light],
    //     view: &Matrix4<f32>,
    //     display: &Display<WindowSurface>,
    //     target: &mut Frame,
    // ) {
    //     if lights.is_empty() {
    //         return;
    //     }
    // // TODO note to self, i changed the shader so it takes a colour and opacity now so need to factor this in

    //     let shader_lights = lights
    //         .iter()
    //         .map(|light| ShaderLight::from(light.clone()))
    //         .collect_vec();

    //     let light_instance_buffer = VertexBuffer::new(display, &shader_lights).unwrap();

    //     let uniforms = uniform! {
    //         vp: maths::raw_matrix(self.perspective_projection * view),
    //     };

    //     target
    //         .draw(
    //             (
    //                 &self.cube_vertex_buffer,
    //                 light_instance_buffer.per_instance().unwrap(),
    //             ),
    //             NoIndices(PrimitiveType::TrianglesList),
    //             &self.light_program,
    //             &uniforms,
    //             &DrawParameters {
    //                 depth: Depth {
    //                     test: DepthTest::IfLess,
    //                     write: true,
    //                     ..Default::default()
    //                 },
    //                 ..DrawParameters::default()
    //             },
    //         )
    //         .unwrap();
    // }

    fn render_model_instances_color(
        &mut self,
        geometry_batches: &GeometryBatches,
        resources: &Resources,
        vp: &[[f32; 4]; 4],
        lights: &[Light],
        camera_position: Point3<f32>,
        display: &Display<WindowSurface>,
        target: &mut Frame,
    ) {
        let camera_position = <[f32; 3]>::from(camera_position);

        let sample_behaviour = SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..SamplerBehavior::default()
        };

        let viewport = self.get_glium_viewport();

        // Draw regular color buffer
        for (key, instances) in geometry_batches.iter() {
            let instance_buffer =
                RendererBuffers::get_vertex_buffer(&mut self.buffers.instance_buffers, key, instances, display);

            let texture = resources.get_texture(key.texture_handle);
            let geometry = resources.get_geometry(key.geometry_handle);

            let uniforms = uniform! {
                vp: *vp,
                camera_position: camera_position,
                // TODO temporary
                light_color: <[f32; 3]>::from(lights.iter().next().unwrap_or(&Light::default()).color.to_rgb_vector3()),
                light_position: <[f32; 3]>::from(lights.iter().next().unwrap_or(&Light::default()).position),
                diffuse_texture: Sampler(&texture.inner_texture, sample_behaviour).0,
                specular_texture: Sampler(&texture.inner_texture, sample_behaviour).0,
            };

            for primitive in geometry.primitives.iter() {
                target
                    .draw(
                        (
                            &primitive.vertex_buffer,
                            instance_buffer
                                .slice(0..instances.len())
                                .unwrap()
                                .per_instance()
                                .unwrap(),
                        ),
                        &primitive.index_buffer,
                        &self.programs.default,
                        &uniforms,
                        &DrawParameters {
                            depth: Depth {
                                test: DepthTest::IfLess,
                                write: true,
                                ..Default::default()
                            },
                            viewport,
                            // backface_culling: BackfaceCullingMode::CullClockwise,
                            ..DrawParameters::default()
                        },
                    )
                    .unwrap();
            }
        }
    }

    fn render_mask_texture(
        &mut self,
        geometry_batches: &GeometryBatches,
        resources: &Resources,
        dimensions: (u32, u32),
        vp: &[[f32; 4]; 4],
        display: &Display<WindowSurface>,
    ) -> Texture2d {
        let mask_texture = Texture2d::empty_with_format(
            display,
            UncompressedFloatFormat::U8,
            MipmapsOption::NoMipmap,
            dimensions.0,
            dimensions.1,
        )
        .unwrap();
        let mut framebuffer = SimpleFrameBuffer::new(display, &mask_texture).unwrap();

        let uniform = uniform! {
            vp: *vp,
        };

        let viewport = self.get_glium_viewport();

        // Only draw selected models into mask
        for (key, instances) in geometry_batches.iter().filter(|(key, _)| key.selected) {
            let instance_buffer =
                RendererBuffers::get_vertex_buffer(&mut self.buffers.instance_buffers, &key, instances, display);

            let geometry = resources.get_geometry(key.geometry_handle);

            for primitive in geometry.primitives.iter() {
                framebuffer
                    .draw(
                        (
                            &primitive.vertex_buffer,
                            instance_buffer
                                .slice(0..instances.len())
                                .unwrap()
                                .per_instance()
                                .unwrap(),
                        ),
                        &primitive.index_buffer,
                        &self.programs.white,
                        &uniform,
                        &DrawParameters {
                            viewport,
                            ..DrawParameters::default()
                        },
                    )
                    .unwrap();
            }
        }

        mask_texture
    }

    fn render_outline(&mut self, outline_texture: Texture2d, target: &mut Frame) {
        let fullscreen_quad_uniforms = uniform! {
            fullscreen_quad_texture: outline_texture,
        };

        // Draw outline
        target
            .draw(
                EmptyVertexAttributes { len: 4 },
                IndicesSource::NoIndices {
                    primitives: PrimitiveType::TriangleStrip,
                },
                &self.programs.fullscreen_quad,
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

    fn render_outline_texture(
        &self,
        mask_texture: Texture2d,
        dimensions: (u32, u32),
        display: &Display<WindowSurface>,
    ) -> Texture2d {
        // Dilate selection mask
        let outline_texture = Texture2d::empty_with_format(
            display,
            UncompressedFloatFormat::U8U8U8U8,
            MipmapsOption::NoMipmap,
            dimensions.0,
            dimensions.1,
        )
        .unwrap();
        let mut outline_framebuffer = SimpleFrameBuffer::new(display, &outline_texture).unwrap();

        let dilate_uniforms = uniform! {
            mask_texture: mask_texture,
            outline_color: <[f32; 3]>::from(colors::SELECTED.to_rgb_vector3()),
            outline_radius: 2
        };

        outline_framebuffer
            .draw(
                EmptyVertexAttributes { len: 4 },
                IndicesSource::NoIndices {
                    primitives: PrimitiveType::TriangleStrip,
                },
                &self.programs.outline,
                &dilate_uniforms,
                &DrawParameters::default(),
            )
            .unwrap();

        outline_texture
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Instance {
    pub transform: [[f32; 4]; 4],
}
implement_vertex!(Instance, transform);

#[derive(Copy, Clone, Debug)]
pub struct SolidColorInstance {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 3],
}
implement_vertex!(SolidColorInstance, transform, color);
