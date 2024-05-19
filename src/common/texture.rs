use glium::glutin::surface::WindowSurface;
use glium::texture::{CompressedTexture2d, RawImage2d};
use glium::Display;
use image::io::Reader;
use image::GenericImageView;
use log::{info};
use memoize::memoize;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct Texture {
    pub inner_texture: CompressedTexture2d,
    pub path: PathBuf,
    pub uuid: Uuid,
}

#[memoize(Ignore: display)]
pub fn load(
    path: PathBuf,
    display: &Display<WindowSurface>,
) -> Result<Arc<Texture>, TextureLoadError> {
    info!("Loading texture {:?}...", path);

    let image =
        Reader::open(path.clone()).map_err(|_| TextureLoadError::ImageNotFound(path.clone()))?;

    let decoded = image
        .decode()
        .map_err(|_| TextureLoadError::UnsupportedImage(path.clone()))?
        .into_rgba32f();

    let image_dimensions = decoded.dimensions();

    let raw_image = RawImage2d::from_raw_rgba_reversed(&decoded.into_raw(), image_dimensions);
    let opengl_texture = CompressedTexture2d::new(display, raw_image).unwrap();

    Ok(Arc::new(Texture {
        inner_texture: opengl_texture,
        path: path.clone(),
        uuid: Uuid::new_v4(),
    }))
}

impl PartialEq<Self> for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for Texture {}

impl Hash for Texture {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state)
    }
}

#[derive(Debug, Clone)]
pub enum TextureLoadError {
    ImageNotFound(PathBuf),
    UnsupportedImage(PathBuf),
}

impl fmt::Display for TextureLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImageNotFound(path) => write!(f, "The image \"{:?}\" could not be found", path),
            Self::UnsupportedImage(path) => {
                write!(f, "The format of the image \"{:?}\" is not supported", path)
            }
        }
    }
}

impl std::error::Error for TextureLoadError {}
