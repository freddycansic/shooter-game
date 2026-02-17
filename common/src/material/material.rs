use crate::engine::resources::TextureHandle;
use nalgebra::Vector3;

pub enum BlendMode {
    // TODO Additive,
    // TODO Multiplicative,
    AlphaBlend { premultiplied: bool },
}

pub struct Opacity {
    pub value: f32,
    pub mode: BlendMode,
}

pub struct Material {
    pub albedo: Vector3<f32>,
    pub albedo_texture: Option<TextureHandle>,
    pub opacity: Option<Opacity>,
}
