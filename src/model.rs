use crate::buffers;
use cgmath::{Matrix4, SquareMatrix};
use color_eyre::Result;
use gltf::buffer::Data;
use gltf::json::accessor::ComponentType;
use gltf::{Accessor, Semantic};
use std::ptr;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

use crate::vertex::Vertex;

pub struct Model {
    pub meshes: Vec<Mesh>,
    // TODO move this to Instance, cause each model will only be in memory once and this is a per instnace thing
    pub model_matrix: Matrix4<f32>,
}

pub struct Primitive {
    pub vertex_buffer: Subbuffer<[Vertex]>,
    pub index_buffer: Subbuffer<[u16]>,
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

impl Model {
    pub fn load(path: &str, memory_allocator: Arc<StandardMemoryAllocator>) -> Result<Self> {
        // TODO parse materials
        let (document, file_buffers, images) = gltf::import(path)?;

        Ok(Model {
            meshes: document
                .meshes()
                .into_iter()
                .map(|mesh| Mesh {
                    primitives: mesh
                        .primitives()
                        .into_iter()
                        .map(|primitive| {
                            Primitive::from(primitive, &file_buffers, memory_allocator.clone())
                                .unwrap()
                        })
                        .collect::<Vec<Primitive>>(),
                })
                .collect::<Vec<Mesh>>(),
            model_matrix: Matrix4::identity(),
        })
    }
}

impl Primitive {
    fn from(
        primitive: gltf::Primitive,
        file_buffers: &[Data],
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self> {
        let num_vertices = primitive.attributes().into_iter().next().unwrap().1.count();
        let num_indices = primitive.indices().expect("No indices? Help, bad.").count();

        assert_ne!(num_vertices, 0, "Empty indices in primitive!");
        assert_ne!(num_indices, 0, "Empty indices in primitive!");

        let mut vertices = vec![Vertex::default(); num_vertices];

        // TODO allow differently sized indices
        let mut indices = vec![0_u16; num_indices];

        for (semantic, accessor) in primitive.attributes() {
            match semantic {
                Semantic::Positions => {
                    const POSITION_OFFSET: usize = 0;
                    // TODO get rid of the explicit Vertex type parameter
                    map_accessor_data_to_buffer::<Vertex, POSITION_OFFSET>(
                        &mut vertices,
                        &accessor,
                        &file_buffers,
                    );
                }
                Semantic::Normals => {
                    // 3 lots of 4 byte floats = position
                    const NORMAL_OFFSET: usize = 3 * 4;
                    map_accessor_data_to_buffer::<Vertex, NORMAL_OFFSET>(
                        &mut vertices,
                        &accessor,
                        &file_buffers,
                    );
                }
                _ => unimplemented!("{:?}", semantic),
            }
        }

        // No offset as indices are scalar
        map_accessor_data_to_buffer::<u16, 0>(
            &mut indices,
            &primitive.indices().unwrap(),
            &file_buffers,
        );

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();

        let index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )
        .unwrap();

        Ok(Primitive {
            vertex_buffer,
            index_buffer,
        })
    }
}

/// Fills the member, specified by the `OFFSET`, of each element of a given buffer from an `Accessor`
fn map_accessor_data_to_buffer<T, const OFFSET: usize>(
    buffer: &mut [T],
    accessor: &Accessor,
    file_buffers: &[Data],
) {
    let buffer_view = accessor
        .view()
        .expect("Sparse accessor not yet implemented HELP");

    let file_buffer = &file_buffers[buffer_view.buffer().index()];

    let byte_stride = buffer_view
        .stride()
        .unwrap_or(calculate_bit_stride(&accessor))
        / 8;

    let positions_start = buffer_view.offset();

    for (index, position_index) in (positions_start..positions_start + buffer_view.length())
        .step_by(byte_stride)
        .enumerate()
    {
        unsafe {
            // Cast to pointer to stop the borrow checker from freaking out then cast to u8
            let current_element_pointer: *mut u8 = &mut buffer[index] as *mut T as *mut u8;
            let member_destination_pointer = current_element_pointer.add(OFFSET);

            // Extract slice from the loaded file buffer
            let member_source_pointer: *const u8 = &file_buffer[position_index];

            ptr::copy(
                member_source_pointer,
                member_destination_pointer,
                byte_stride,
            );
        }
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
