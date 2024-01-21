use crate::{maths, shaders};
use cgmath::{Matrix, Matrix4, SquareMatrix};
use color_eyre::Result;
use gltf::buffer::Data;
use gltf::json::accessor::ComponentType;
use gltf::{Accessor, Semantic};
use itertools::Itertools;
use log::{debug, warn};
use std::fmt::Debug;
use std::mem::offset_of;
use std::ptr;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, Subbuffer};
use vulkano::command_buffer::RecordingCommandBuffer;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};

use crate::buffers;
use crate::context::Allocators;
use crate::texture::Texture;
use crate::vertex::Vertex;

pub struct ModelInstance {
    pub model: Arc<Model>,
    pub transform: Matrix4<f32>,
}

impl From<Arc<Model>> for ModelInstance {
    fn from(model: Arc<Model>) -> Self {
        Self {
            model,
            transform: Matrix4::identity(),
        }
    }
}

impl ModelInstance {
    pub fn render(
        &self,
        builder: &mut RecordingCommandBuffer,
        allocators: &Allocators,
        pipeline: Arc<GraphicsPipeline>,
        // TODO temporary
        texture: &Texture,
    ) {
        let model_normal_uniform_subbuffer = {
            let uniform_data = shaders::vs::ModelUniform {
                model: self.transform,
                normal: self.transform.invert().unwrap().transpose(),
            };

            buffers::create_subbuffer(&allocators.subbuffer_allocator, uniform_data)
        };

        let per_primitive_descriptor_set = DescriptorSet::new(
            allocators.descriptor_set_allocator.clone(),
            pipeline.layout().set_layouts()[1].clone(),
            [
                WriteDescriptorSet::buffer(0, model_normal_uniform_subbuffer),
                WriteDescriptorSet::image_view_sampler(
                    1,
                    texture.image_view.clone(),
                    texture.sampler.clone(),
                ),
            ],
            [],
        )
        .unwrap();

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                1,
                per_primitive_descriptor_set,
            )
            .unwrap()
            .bind_vertex_buffers(0, self.model.meshes[0].primitives[0].vertex_buffer.clone())
            .unwrap()
            .bind_index_buffer(self.model.meshes[0].primitives[0].index_buffer.clone())
            .unwrap();

        unsafe {
            builder
                .draw_indexed(
                    self.model.meshes[0].primitives[0].index_buffer.len() as u32,
                    1,
                    0,
                    0,
                    0,
                )
                .unwrap();
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Primitive {
    pub vertex_buffer: Subbuffer<[Vertex]>,
    pub index_buffer: Subbuffer<[u16]>,
}

// TODO could move all vertices / indices into one buffer and then have an offset into this for each primitive
pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

impl Model {
    pub fn load(path: &str, memory_allocator: Arc<StandardMemoryAllocator>) -> Result<Arc<Self>> {
        debug!("Loading model \"{path}\"...");

        // TODO parse materials
        let (document, file_buffers, _images) = gltf::import(path)?;

        Ok(Arc::new(Model {
            meshes: document
                .meshes()
                .map(|mesh| Mesh {
                    name: mesh.name().map(str::to_owned),
                    primitives: mesh
                        .primitives()
                        .map(|primitive| {
                            Primitive::from(primitive, &file_buffers, memory_allocator.clone())
                                .unwrap()
                        })
                        .collect::<Vec<Primitive>>(),
                })
                .collect::<Vec<Mesh>>(),
        }))
    }

    pub fn render(&self) {
        for mesh in &self.meshes {
            mesh.render();
        }
    }
}

impl Mesh {
    pub fn render(&self) {
        for primitive in &self.primitives {
            primitive.render();
        }
    }
}

impl Primitive {
    fn render(&self) {
        // build uniforms

        // render
    }

    fn from(
        primitive: gltf::Primitive,
        file_buffers: &[Data],
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self> {
        let available_attributes = primitive
            .attributes()
            .map(|(semantic, _)| semantic)
            .collect_vec();

        debug!("Available attributes: {available_attributes:?}");

        assert!(
            available_attributes.contains(&Semantic::Positions),
            "No position data for primitive!"
        );

        let mut vertices = Self::extract_vertices(&primitive, file_buffers);
        let indices = Self::extract_indices(&primitive, file_buffers);

        // TODO understand tex coord set index
        if !available_attributes.contains(&Semantic::TexCoords(0)) {
            warn!("Mesh primitive does include texture coordinates! Generating...");
            generate_tex_coords(&mut vertices);
        }

        let vertex_buffer = buffers::create_mapped_buffer_from_iter(
            memory_allocator.clone(),
            BufferUsage::VERTEX_BUFFER,
            MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            vertices,
        )?;

        let index_buffer = buffers::create_mapped_buffer_from_iter(
            memory_allocator.clone(),
            BufferUsage::INDEX_BUFFER,
            MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            indices,
        )?;

        Ok(Primitive {
            vertex_buffer,
            index_buffer,
        })
    }

    fn extract_indices(primitive: &gltf::Primitive, file_buffers: &[Data]) -> Vec<u16> {
        let num_indices = primitive.indices().expect("No indices? Help, bad.").count();
        // TODO allow differently sized indices
        let mut indices = vec![0_u16; num_indices];

        map_accessor_data_to_buffer(
            &mut indices,
            // No offset as indices are scalar
            0,
            &primitive.indices().unwrap(),
            file_buffers,
        );

        indices
    }

    fn extract_vertices(primitive: &gltf::Primitive, file_buffers: &[Data]) -> Vec<Vertex> {
        let num_vertices = primitive.attributes().next().unwrap().1.count();
        let mut vertices = vec![Vertex::default(); num_vertices];

        for (semantic, accessor) in primitive.attributes() {
            match semantic {
                Semantic::Positions => {
                    map_accessor_data_to_buffer(
                        &mut vertices,
                        offset_of!(Vertex, position),
                        &accessor,
                        file_buffers,
                    );
                }
                Semantic::Normals => {
                    map_accessor_data_to_buffer(
                        &mut vertices,
                        offset_of!(Vertex, normal),
                        &accessor,
                        file_buffers,
                    );
                }
                Semantic::TexCoords(0) => {
                    map_accessor_data_to_buffer(
                        &mut vertices,
                        offset_of!(Vertex, tex_coord),
                        &accessor,
                        file_buffers,
                    );
                }
                _ => unimplemented!("{semantic:?}"),
            }
        }

        vertices
    }
}

/// Fills the member, specified by the `byte_offset`, of each element of a given buffer from an `Accessor`
fn map_accessor_data_to_buffer<T: Debug>(
    destination_buffer: &mut [T],
    byte_offset: usize,
    accessor: &Accessor,
    file_buffers: &[Data],
) {
    let buffer_view = accessor
        .view()
        .expect("Sparse accessor not yet implemented HELP");

    let file_buffer = &file_buffers[buffer_view.buffer().index()];

    let byte_stride = buffer_view
        .stride()
        .unwrap_or(calculate_bit_stride(accessor))
        / 8;

    let file_buffer_offset = buffer_view.offset();

    for (index, element_start_index) in (file_buffer_offset
        ..file_buffer_offset + buffer_view.length())
        .step_by(byte_stride)
        .enumerate()
    {
        unsafe {
            // Cast to pointer to stop the borrow checker from freaking out then cast to u8
            let current_destination_pointer: *mut u8 =
                &mut destination_buffer[index] as *mut T as *mut u8;
            let member_destination_pointer = current_destination_pointer.add(byte_offset);

            // Extract slice from the loaded file buffer
            let member_source_pointer: *const u8 = &file_buffer[element_start_index];

            ptr::copy(
                member_source_pointer,
                member_destination_pointer,
                byte_stride,
            );
        }
    }
}

fn generate_tex_coords(vertices: &mut [Vertex]) {
    let mut x_min = f32::MAX;
    let mut x_max = f32::MIN;
    let mut z_min = f32::MAX;
    let mut z_max = f32::MIN;

    for vertex in vertices.iter() {
        x_min = x_min.min(vertex.position[0]);
        x_max = x_max.max(vertex.position[0]);

        z_min = z_min.min(vertex.position[2]);
        z_max = x_max.max(vertex.position[2]);
    }

    // project texture coordinates on to xz plane over primitive
    for vertex in vertices.iter_mut() {
        let x_tex_coord = maths::linear_map(vertex.position[0], x_min, x_max, 0.0, 1.0);
        let y_tex_coord = maths::linear_map(vertex.position[2], z_min, z_max, 0.0, 1.0);

        vertex.tex_coord = [x_tex_coord, y_tex_coord];
    }
}

fn calculate_bit_stride(accessor: &Accessor) -> usize {
    let component_size = match accessor.data_type() {
        ComponentType::U8 | ComponentType::I8 => 8,
        ComponentType::U16 | ComponentType::I16 => 16,
        ComponentType::U32 | ComponentType::F32 => 32,
    };

    accessor.dimensions().multiplicity() * component_size
}
