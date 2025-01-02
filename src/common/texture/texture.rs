use color_eyre::eyre::Result;
use glium::texture::RawImage2d;
use image::ImageReader;
use log::info;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TextureLoadError {
    ImageNotFound(PathBuf),
    UnsupportedImage(PathBuf),
    CreateTextureError(glium::texture::TextureCreationError),
    CubemapDimensionError(HashSet<(u32, u32)>),
    CubemapFramebufferError,
}

impl fmt::Display for TextureLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImageNotFound(path) => write!(f, "The image {:?} could not be found", path),
            Self::UnsupportedImage(path) => {
                write!(f, "The format of the image {:?} is not supported", path)
            }
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

pub fn load_raw_image<'a>(path: &PathBuf) -> Result<RawImage2d<'a, f32>, TextureLoadError> {
    info!("Loading texture {:?}...", path);

    let image = ImageReader::open(path.clone())
        .map_err(|_| TextureLoadError::ImageNotFound(path.clone()))?;

    let decoded = image
        .decode()
        .map_err(|_| TextureLoadError::UnsupportedImage(path.clone()))?
        .into_rgba32f();

    let image_dimensions = decoded.dimensions();

    Ok(RawImage2d::from_raw_rgba(
        decoded.into_raw(),
        image_dimensions,
    ))
}
