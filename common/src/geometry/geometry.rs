use std::fmt::Debug;
use std::path::PathBuf;

use color_eyre::Result;
use color_eyre::eyre::Context;
use glium::glutin::surface::WindowSurface;
use glium::index::PrimitiveType;
use glium::{Display, IndexBuffer, VertexBuffer};
use gltf::buffer::Data;
use itertools::Itertools;

use crate::geometry::GeometryVertex;

use crate::geometry::Primitive;
use crate::ui;

#[derive(Debug)]
pub struct Geometry {
    pub name: String,
    pub primitives: Vec<Primitive>,
}

impl Geometry {
    pub fn load(path: PathBuf, display: &Display<WindowSurface>) -> Result<Vec<Geometry>> {
        log::info!("Loading gltf {:?}...", path);

        let (document, file_buffers, _images) = gltf::import(&path)
            .context(format!("The model \"{:?}\" does not exist", path.clone()))?;

        let models = document
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
                            path.clone(),
                        )
                    })
                    .collect::<Result<Vec<Primitive>>>()?;

                Ok(Geometry {
                    name: mesh
                        .name()
                        .unwrap_or(ui::default_name::model().as_str())
                        .to_owned(),
                    primitives,
                })
            })
            .collect::<Result<Vec<Geometry>>>()?;

        Ok(models)
    }
}

impl Primitive {
    pub fn from_gltf_primitive(
        primitive: gltf::Primitive,
        file_buffers: &[Data],
        display: &Display<WindowSurface>,
        _path: PathBuf,
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

        vertices.extend(positions.zip_eq(normals).zip_eq(tex_coords).map(
            |((position, normal), tex_coord)| GeometryVertex {
                position,
                normal,
                tex_coord,
            },
        ));

        let vertex_buffer = VertexBuffer::new(display, &vertices).unwrap();

        let index_buffer =
            IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).unwrap();

        Ok(Primitive {
            vertex_buffer,
            index_buffer,
            vertices,
            indices,
        })
    }
}
