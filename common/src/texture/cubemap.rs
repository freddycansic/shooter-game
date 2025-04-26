use fxhash::FxHashSet;
use std::path::PathBuf;
use std::sync::Arc;

use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use glium::texture::CubeLayer;
use glium::uniforms::MagnifySamplerFilter;
use glium::{BlitTarget, Display, Surface, Texture2d};
use memoize::memoize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::texture::texture;
use crate::texture::texture::TextureLoadError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cubemap {
    pub uuid: Uuid,
    pub directory: PathBuf,

    #[serde(skip)]
    pub inner_cubemap: Option<glium::texture::Cubemap>,
}

impl Cubemap {
    pub fn load(
        directory: PathBuf,
        display: &Display<WindowSurface>,
    ) -> color_eyre::Result<Arc<Self>> {
        Ok(load(directory, display)?)
    }
}

impl PartialEq<Self> for Cubemap {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

#[memoize(Ignore: display)]
fn load(
    directory: PathBuf,
    display: &Display<WindowSurface>,
) -> Result<Arc<Cubemap>, TextureLoadError> {
    let side_names = vec!["posx", "negx", "posy", "negy", "posz", "negz"];
    let cube_layers = vec![
        CubeLayer::PositiveX,
        CubeLayer::NegativeX,
        CubeLayer::PositiveY,
        CubeLayer::NegativeY,
        CubeLayer::PositiveZ,
        CubeLayer::NegativeZ,
    ];

    // Load each side of cubemap
    let mut textures = side_names
        .into_iter()
        .map(|side| {
            let mut path = directory.clone();
            path.push(side);

            let raw_image = texture::load_raw_image(&path.with_extension("jpg"))?;

            Texture2d::new(display, raw_image).map_err(TextureLoadError::CreateTextureError)
        })
        .collect::<Result<Vec<Texture2d>, TextureLoadError>>()?;

    // Check each side is of the same dimension
    let unique_cubemap_dimensions = FxHashSet::from_iter(
        textures
            .iter()
            .map(|texture| (texture.width(), texture.height())),
    );

    if unique_cubemap_dimensions.len() > 1 {
        return Err(TextureLoadError::CubemapDimensionError(
            unique_cubemap_dimensions,
        ));
    }

    let dimension = textures[0].width();

    // Create cubemap texture and framebuffers
    let inner_cubemap = glium::texture::Cubemap::empty(display, dimension)
        .map_err(TextureLoadError::CreateTextureError)?;

    let framebuffers = cube_layers
        .into_iter()
        .map(|cube_layer| {
            SimpleFrameBuffer::new(display, inner_cubemap.main_level().image(cube_layer))
        })
        .collect::<Result<Vec<SimpleFrameBuffer>, _>>()
        .map_err(|_| TextureLoadError::CubemapFramebufferError)?;

    let blit_target = BlitTarget {
        left: 0,
        bottom: 0,
        width: dimension as i32,
        height: dimension as i32,
    };

    // Blit each texture on to its framebuffer
    textures
        .iter_mut()
        .zip(framebuffers.iter())
        .for_each(|(texture, framebuffer)| {
            texture.as_surface().blit_whole_color_to(
                framebuffer,
                &blit_target,
                MagnifySamplerFilter::Linear,
            )
        });

    Ok(Arc::new(Cubemap {
        inner_cubemap: Some(inner_cubemap),
        directory: directory.clone(),
        uuid: Uuid::new_v4(),
    }))
}
