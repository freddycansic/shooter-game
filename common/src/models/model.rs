use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use color_eyre::Result;
use fxhash::FxHasher;
use glium::glutin::surface::WindowSurface;
use glium::index::PrimitiveType;
use glium::{Display, IndexBuffer, VertexBuffer};
use gltf::buffer::Data;
use itertools::Itertools;
use uuid::Uuid;

use crate::import::gltf::ModelLoadError;
use crate::models::model_vertex::ModelVertex;

use super::Primitive;

#[derive(Debug)]
pub struct Model {
    pub uuid: Uuid,
    pub name: String,
    pub primitives: Vec<Primitive>,
}

impl PartialEq<Self> for Model {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for Model {}

impl Hash for Model {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut hasher = FxHasher::default();
        self.uuid.hash(&mut hasher);

        let result = hasher.finish();
        state.write_u64(result);
    }
}

impl Primitive {
    pub fn from_gltf_primitive(
        primitive: gltf::Primitive,
        file_buffers: &[Data],
        display: &Display<WindowSurface>,
        path: PathBuf,
    ) -> Result<Self, ModelLoadError> {
        let reader = primitive.reader(|buffer| Some(&file_buffers[buffer.index()].0));

        let positions = reader
            .read_positions()
            .ok_or(ModelLoadError::NoPositions(path.clone()))?;
        let normals = reader.read_normals().unwrap();

        // Primitives can have multiple "sets" of texture coordinates which can differ on whether they are being used for diffuse maps, specular etc.
        // 0 is the standard place for diffuse maps
        let tex_coords = reader.read_tex_coords(0).unwrap().into_f32();

        let indices = reader
            .read_indices()
            .ok_or(ModelLoadError::NoIndices(path.clone()))?
            .into_u32()
            .collect_vec();

        if reader.read_tex_coords(1).is_some() {
            log::warn!("There exists more than one set of texture coords for this primitive");
        }

        let num_vertices = primitive.attributes().next().unwrap().1.count();
        let mut vertices = Vec::<ModelVertex>::with_capacity(num_vertices);

        vertices.extend(positions.zip_eq(normals).zip_eq(tex_coords).map(
            |((position, normal), tex_coord)| ModelVertex {
                position,
                normal,
                tex_coord,
            },
        ));

        let vertex_buffer = VertexBuffer::new(display, &vertices)
            .map_err(|_| ModelLoadError::CreateBufferError(path.clone()))?;

        let index_buffer = IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices)
            .map_err(|_| ModelLoadError::CreateBufferError(path))?;

        Ok(Primitive {
            vertex_buffer,
            index_buffer,
            vertices,
            indices,
        })
    }
}
