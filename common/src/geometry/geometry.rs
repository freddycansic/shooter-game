use std::fmt::Debug;
use std::path::{Path, PathBuf};

use crate::collision::colliders::bvh::Bvh;
use crate::geometry::GeometryVertex;
use color_eyre::eyre::Context;
use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::index::PrimitiveType;
use glium::{Display, IndexBuffer, VertexBuffer};
use gltf::buffer::Data;
use itertools::Itertools;

use crate::geometry::primitive::{PrimitiveCpu, PrimitiveGpu};
use crate::geometry::Primitive;

#[derive(Debug)]
pub struct Geometry {
    pub name: String,
    pub primitives: Vec<Primitive>,
    pub bvh: Bvh,
}

impl Geometry {
    pub fn load(path: &Path, display: Option<&Display<WindowSurface>>) -> Result<Vec<Geometry>> {
        log::info!("Loading gltf {:?}...", path);

        let (document, file_buffers, _images) =
            gltf::import(&path).context(format!("The model \"{:?}\" does not exist", path.clone()))?;

        let models = document
            .meshes()
            .enumerate()
            .map(|(mesh_index, mesh)| {
                let primitives = mesh
                    .primitives()
                    .enumerate()
                    .map(|(primitive_index, primitive)| {
                        log::debug!("Loading mesh {} primitive {}", mesh_index, primitive_index);

                        Primitive::from_gltf_primitive(primitive, &file_buffers, display)
                    })
                    .collect::<Result<Vec<Primitive>>>()?;

                let bvh = Bvh::from_primitives(&primitives);

                Ok(Geometry {
                    name: mesh.name().unwrap_or("Unnamed Geometry").to_owned(),
                    primitives,
                    bvh,
                })
            })
            .collect::<Result<Vec<Geometry>>>()?;

        Ok(models)
    }
}

impl Primitive {
    fn from_gltf_primitive(
        primitive: gltf::Primitive,
        file_buffers: &[Data],
        display: Option<&Display<WindowSurface>>,
    ) -> Result<Self> {
        let reader = primitive.reader(|buffer| Some(&file_buffers[buffer.index()].0));

        let positions = reader.read_positions().unwrap();
        let normals = reader.read_normals().unwrap();

        // Primitives can have multiple "sets" of texture coordinates which can differ on whether they are being used for diffuse maps, specular etc.
        // 0 is the standard place for diffuse maps
        let tex_coords = reader.read_tex_coords(0).unwrap().into_f32();

        let indices = reader.read_indices().unwrap().into_u32().collect_vec();

        if reader.read_tex_coords(1).is_some() {
            log::warn!("There exists more than one set of texture coords for this primitive");
        }

        let num_vertices = primitive.attributes().next().unwrap().1.count();
        let mut vertices = Vec::<GeometryVertex>::with_capacity(num_vertices);

        vertices.extend(
            positions
                .zip_eq(normals)
                .zip_eq(tex_coords)
                .map(|((position, normal), tex_coord)| GeometryVertex {
                    position,
                    normal,
                    tex_coord,
                }),
        );

        let primitive_gpu = display.map(|display| PrimitiveGpu {
            vertex_buffer: VertexBuffer::new(display, &vertices).unwrap(),
            index_buffer: IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap(),
        });

        Ok(Primitive {
            cpu: PrimitiveCpu { vertices, indices },
            gpu: primitive_gpu,
        })
    }
}
