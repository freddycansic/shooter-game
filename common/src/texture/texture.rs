use std::path::PathBuf;

use color_eyre::eyre::Result;
use glium::texture::RawImage2d;

use crate::import;

pub fn load_raw_image<'a>(path: &PathBuf) -> Result<RawImage2d<'a, u8>> {
    let rgba8 = import::image::load_dynamic_image(path)?.into_rgba8();

    let dimensions = rgba8.dimensions();

    Ok(RawImage2d::from_raw_rgba(rgba8.into_raw(), dimensions))
}
