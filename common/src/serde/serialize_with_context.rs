use glium::{Display, glutin::surface::WindowSurface};

use crate::resources::Resources;

pub trait SerializeWithContext {
    type Serialized;

    fn serialize_with(&self, resources: &Resources) -> Self::Serialized;
    fn deserialize_with(
        serialized: Self::Serialized,
        display: &Display<WindowSurface>,
        resources: &mut Resources,
    ) -> Self;
}
