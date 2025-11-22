// use std::sync::Arc;

// use color_eyre::eyre::Result;
// use glium::Display;
// use glium::glutin::surface::WindowSurface;
// use serde::{Deserialize, Serialize};
// use uuid::Uuid;

// use crate::texture::Texture2D;

// #[derive(Serialize, Deserialize, Clone, Eq, Hash, PartialEq, Debug)]
// pub struct Material {
//     pub uuid: Uuid,
//     pub diffuse: Texture2D,
//     pub specular: Texture2D,
// }

// // TODO load memoization
// impl Material {
//     pub fn default(display: &Display<WindowSurface>) -> Result<Arc<Self>> {
//         let default_diffuse = Texture2D::default_diffuse(display)?;
//         let (width, height) = default_diffuse.inner_texture.as_ref().unwrap().dimensions();

//         Ok(Arc::new(Self {
//             uuid: Uuid::new_v4(),
//             diffuse: default_diffuse,
//             specular: Texture2D::solid(width, height, display)?,
//         }))
//     }
// }
