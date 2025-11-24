use color_eyre::eyre::Result;
use fxhash::FxHashSet;
use std::path::PathBuf;
use std::sync::Arc;

use glium::framebuffer::SimpleFrameBuffer;
use glium::glutin::surface::WindowSurface;
use glium::texture::CubeLayer;
use glium::uniforms::MagnifySamplerFilter;
use glium::{BlitTarget, Display, Surface, Texture2d};

use crate::texture::texture;

pub struct Cubemap {
    pub directory: PathBuf,
    pub inner_cubemap: glium::texture::Cubemap,
}

impl Cubemap {
    pub fn load(directory: PathBuf, display: &Display<WindowSurface>) -> Result<Self> {
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

                let raw_image = texture::load_raw_image(&path.with_extension("jpg")).unwrap();

                Texture2d::new(display, raw_image).unwrap()
            })
            .collect::<Vec<Texture2d>>();

        // Check each side is of the same dimension
        let unique_cubemap_dimensions = FxHashSet::from_iter(
            textures
                .iter()
                .map(|texture| (texture.width(), texture.height())),
        );

        if unique_cubemap_dimensions.len() > 1 {
            panic!(
                "Cubemap sides must be the same size, found sizes: {:?}",
                unique_cubemap_dimensions
            );
        }

        let dimension = textures[0].width();

        // Create cubemap texture and framebuffers
        let inner_cubemap = glium::texture::Cubemap::empty(display, dimension).unwrap();

        let framebuffers = cube_layers
            .into_iter()
            .map(|cube_layer| {
                SimpleFrameBuffer::new(display, inner_cubemap.main_level().image(cube_layer))
                    .unwrap()
            })
            .collect::<Vec<SimpleFrameBuffer>>();

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

        Ok(Cubemap {
            inner_cubemap,
            directory: directory.clone(),
        })
    }
}
