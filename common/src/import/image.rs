use std::path::{Path, PathBuf};

use color_eyre::Result;
use image::{DynamicImage, ImageReader};
use log::info;

#[derive(Debug, Clone)]
pub enum ImageLoadError {
    ImageNotFound(PathBuf),
    UnsupportedImage(PathBuf),
}

impl std::fmt::Display for ImageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImageNotFound(path) => write!(f, "The image {:?} could not be found", path),
            Self::UnsupportedImage(path) => {
                write!(f, "The format of the image {:?} is not supported", path)
            }
        }
    }
}

impl std::error::Error for ImageLoadError {}

pub fn load_dynamic_image(path: &Path) -> Result<DynamicImage, ImageLoadError>
where
{
    info!("Loading image {:?}", path);

    let image =
        ImageReader::open(path).map_err(|_| ImageLoadError::ImageNotFound(path.to_path_buf()))?;

    let decoded = image
        .decode()
        .map_err(|_| ImageLoadError::UnsupportedImage(path.to_path_buf()))?;

    Ok(decoded)
}
