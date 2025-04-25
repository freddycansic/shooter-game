use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::mem::offset_of;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fmt, ptr};

use color_eyre::Result;
use color_eyre::eyre::ContextCompat;
use fxhash::FxHasher;
use glium::glutin::surface::WindowSurface;
use glium::index::PrimitiveType;
use glium::{Display, IndexBuffer, VertexBuffer};
use gltf::buffer::Data;
use gltf::json::accessor::ComponentType;
use gltf::mesh::Reader;
use gltf::mesh::util::ReadIndices;
use gltf::{Accessor, Semantic};
use itertools::Itertools;
use log::{debug, info, warn};
use memoize::memoize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::maths;
use crate::models::model_vertex::ModelVertex;

#[derive(Debug)]
pub struct Primitive {
    pub vertex_buffer: VertexBuffer<ModelVertex>,
    pub index_buffer: IndexBuffer<u32>,
}

// TODO could move all vertices / indices into one buffer and then have an offset into this for each primitive
#[derive(Debug)]
pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

#[derive(Debug, Clone)]
pub enum ModelLoadError {
    ModelDoesNotExist(PathBuf),
    CreateBufferError(PathBuf),
    NoPositions(PathBuf),
    NoIndices(PathBuf),
}

impl std::error::Error for ModelLoadError {}

impl fmt::Display for ModelLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ModelDoesNotExist(path) => {
                write!(f, "The model \"{:?}\" does not exist", path)
            }
            Self::CreateBufferError(path) => {
                write!(f, "Could not create buffers for the model \"{:?}\"", path)
            }
            Self::NoPositions(path) => {
                write!(
                    f,
                    "Could not extract primitive vertex positions for the model {:?}",
                    path
                )
            }
            Self::NoIndices(path) => {
                write!(
                    f,
                    "Could not extract primitive indices for the model {:?}",
                    path
                )
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    #[serde(with = "crate::serde::uuid")]
    pub uuid: Uuid,
    pub path: PathBuf,
    #[serde(skip)]
    // This is in a mutex for interior mutability
    // TODO figure out how to make this not like this
    pub meshes: Mutex<Option<Vec<Mesh>>>,
}

impl Model {
    pub fn load(
        path: PathBuf,
        display: &Display<WindowSurface>,
    ) -> Result<Arc<Self>, ModelLoadError> {
        load(path, display)
    }

    pub fn load_meshes(&self, display: &Display<WindowSurface>) -> Result<(), ModelLoadError> {
        // TODO parse materials
        let (document, file_buffers, _images) = gltf::import(&self.path)
            .map_err(|_| ModelLoadError::ModelDoesNotExist(self.path.clone()))?;

        let meshes = document
            .meshes()
            .enumerate()
            .map(|(mesh_index, mesh)| {
                let primitives = mesh
                    .primitives()
                    .enumerate()
                    .map(|(primitive_index, primitive)| {
                        log::debug!("Loading mesh {} primitive {}", mesh_index, primitive_index);

                        Primitive::from_gltf_primitive(
                            primitive,
                            &file_buffers,
                            display,
                            self.path.clone(),
                        )
                    })
                    .collect::<Result<Vec<Primitive>, ModelLoadError>>()?;

                Ok(Mesh {
                    name: mesh.name().map(str::to_owned),
                    primitives,
                })
            })
            .collect::<Result<Vec<Mesh>, ModelLoadError>>()?;

        *self.meshes.lock().unwrap() = Some(meshes);

        Ok(())
    }
}

#[memoize(Ignore: display)]
fn load(path: PathBuf, display: &Display<WindowSurface>) -> Result<Arc<Model>, ModelLoadError> {
    info!("Loading models {:?}...", path);

    let model = Model {
        uuid: Uuid::new_v4(),
        path: path.clone(),
        meshes: Mutex::new(None),
    };

    model.load_meshes(display)?;

    Ok(Arc::new(model))
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
    fn from_gltf_primitive(
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
        })
    }
}
