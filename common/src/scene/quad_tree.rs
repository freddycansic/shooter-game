use std::path::PathBuf;

use fxhash::{FxBuildHasher, FxHashMap};
use glium::{Display, glutin::surface::WindowSurface};
use itertools::Itertools;
use nalgebra::{Point2, Vector2};
use serde::{Deserialize, Serialize};

use crate::{
    quad::{Quad, QuadVertex},
    resources::{Resources, TextureHandle},
    serde::SerializeWithContext,
};

pub struct QuadTree(pub Vec<Vec<Quad>>);

pub type QuadBatches = FxHashMap<TextureHandle, Vec<QuadVertex>>;

impl QuadTree {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn batch(&mut self) -> QuadBatches {
        let mut batches = QuadBatches::with_hasher(FxBuildHasher::new());

        for (layer, quads) in self.0.iter().enumerate() {
            for quad in quads {
                let batch = batches.entry(quad.texture).or_insert(vec![]);

                batch.push(QuadVertex {
                    position: [quad.position.x, quad.position.y],
                    size: [quad.size.x, quad.size.y],
                    layer: layer as i32,
                });
            }
        }

        batches
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedQuadTree(Vec<Vec<SerializedQuad>>);

impl SerializeWithContext for QuadTree {
    type Serialized = SerializedQuadTree;

    fn serialize_with(&self, resources: &Resources) -> Self::Serialized {
        SerializedQuadTree(
            self.0
                .iter()
                .map(|quads| quads.iter().map(|quad| quad.serialize_with(resources)).collect_vec())
                .collect_vec(),
        )
    }

    fn deserialize_with(
        serialized: Self::Serialized,
        display: &Display<WindowSurface>,
        resources: &mut Resources,
    ) -> Self {
        QuadTree(
            serialized
                .0
                .into_iter()
                .map(|quads| {
                    quads
                        .into_iter()
                        .map(|quad| Quad::deserialize_with(quad, display, resources))
                        .collect_vec()
                })
                .collect_vec(),
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedQuad {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub texture: PathBuf,

    pub selected: bool,
    pub name: String,
}

impl SerializeWithContext for Quad {
    type Serialized = SerializedQuad;

    fn serialize_with(&self, resources: &Resources) -> Self::Serialized {
        let texture_path = resources.get_texture_path(self.texture);

        Self::Serialized {
            position: self.position,
            size: self.size,
            texture: texture_path,

            selected: self.selected,
            name: self.name.clone(),
        }
    }

    fn deserialize_with(
        serialized: Self::Serialized,
        display: &Display<WindowSurface>,
        resources: &mut Resources,
    ) -> Self {
        let texture_handle = resources.get_texture_handle(&serialized.texture, display).unwrap();

        Quad {
            position: serialized.position,
            size: serialized.size,
            texture: texture_handle,

            selected: serialized.selected,
            name: serialized.name.clone(),
        }
    }
}
