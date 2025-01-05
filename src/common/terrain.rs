use crate::import;
use cgmath::Vector3;
use color_eyre::eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::{implement_vertex, Display, VertexBuffer};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Terrain {
    pub path: PathBuf,
    // pub heightmap: Vec<Vec<u16>>,
    #[serde(skip)]
    pub vertex_buffer: Option<VertexBuffer<TerrainVertex>>,
}

#[derive(Copy, Clone)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
implement_vertex!(TerrainVertex, position, normal);

impl Terrain {
    pub fn load(path: &Path, display: &Display<WindowSurface>) -> Result<Self> {
        let image_1d = import::image::load_dynamic_image(path)?.into_luma16();

        let dimensions = image_1d.dimensions();

        let mut heightmap = Vec::with_capacity(dimensions.0 as usize);
        for row in image_1d.rows() {
            heightmap.push(row.map(|pixel| pixel.0[0]).collect_vec())
        }

        let mut vertices = Vec::with_capacity(dimensions.0 as usize * dimensions.1 as usize);
        for col in 0..heightmap.len() - 1 {
            for row in 0..heightmap[0].len() - 1 {
                let scale = 30.0;

                let height = heightmap[col][row] as f32 / u16::MAX as f32 * scale;
                let height_right = heightmap[col + 1][row] as f32 / u16::MAX as f32 * scale;
                let height_below = heightmap[col][row + 1] as f32 / u16::MAX as f32 * scale;
                let height_right_below =
                    heightmap[col + 1][row + 1] as f32 / u16::MAX as f32 * scale;

                let offset = Vector3::new(
                    -(heightmap.len() as f32 / 2.0),
                    -scale,
                    -(heightmap[0].len() as f32 / 2.0),
                );

                let position = Vector3::new(col as f32, height, row as f32) + offset;
                let position_right =
                    Vector3::new(col as f32 + 1.0, height_right, row as f32) + offset;
                let position_below =
                    Vector3::new(col as f32, height_below, row as f32 + 1.0) + offset;
                let position_right_below =
                    Vector3::new(col as f32 + 1.0, height_right_below, row as f32 + 1.0) + offset;

                let triangle_1_perp_1 = position_right - position;
                let triangle_1_perp_2 = position_below - position;
                let triangle_1_normal = -triangle_1_perp_1.cross(triangle_1_perp_2);

                let triangle_2_perp_1 = position_right - position_right_below;
                let triangle_2_perp_2 = position_below - position_right_below;
                let triangle_2_normal = triangle_2_perp_1.cross(triangle_2_perp_2);

                vertices.push(TerrainVertex {
                    position: position.into(),
                    normal: triangle_1_normal.into(),
                });
                vertices.push(TerrainVertex {
                    position: position_right.into(),
                    normal: triangle_1_normal.into(),
                });
                vertices.push(TerrainVertex {
                    position: position_below.into(),
                    normal: triangle_1_normal.into(),
                });

                vertices.push(TerrainVertex {
                    position: position_right.into(),
                    normal: triangle_2_normal.into(),
                });
                vertices.push(TerrainVertex {
                    position: position_right_below.into(),
                    normal: triangle_2_normal.into(),
                });
                vertices.push(TerrainVertex {
                    position: position_below.into(),
                    normal: triangle_2_normal.into(),
                });
            }
        }

        let vertex_buffer = VertexBuffer::immutable(display, &vertices)?;

        Ok(Self {
            path: path.to_path_buf(),
            // heightmap,
            vertex_buffer: Some(vertex_buffer),
        })
    }
}
