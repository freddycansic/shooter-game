use crate::import;
use crate::import::image::ImageLoadError;
use color_eyre::eyre::Result;
use glium::texture::RawImage2d;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TextureLoadError {
    ImageLoadError(ImageLoadError),
    CreateTextureError(glium::texture::TextureCreationError),
    CubemapDimensionError(HashSet<(u32, u32)>),
    CubemapFramebufferError,
}

impl fmt::Display for TextureLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImageLoadError(err) => write!(f, "{}", err),
            Self::CreateTextureError(err) => write!(f, "Failed to create texture: {}", err),
            Self::CubemapDimensionError(dimensions) => write!(
                f,
                "Cubemap sides must be the same size, found sizes: {:?}",
                dimensions
            ),
            Self::CubemapFramebufferError => {
                write!(f, "Could not create framebuffer(s) when creating cubemap")
            }
        }
    }
}

impl std::error::Error for TextureLoadError {}

pub fn load_raw_image<'a>(path: &PathBuf) -> Result<RawImage2d<'a, u8>, TextureLoadError> {
    let rgba8 = import::image::load_dynamic_image(path)
        .map_err(TextureLoadError::ImageLoadError)?
        .into_rgba8();

    let dimensions = rgba8.dimensions();

    Ok(RawImage2d::from_raw_rgba(rgba8.into_raw(), dimensions))
}
