use glium::{glutin::surface::WindowSurface, Display};

use crate::engine::assets::Assets;

pub trait SerializeWithContext {
    type Serialized;

    fn serialize_with(&self, assets: &Assets) -> Self::Serialized;
    fn deserialize_with(serialized: Self::Serialized, display: &Display<WindowSurface>, assets: &mut Assets) -> Self;
}
