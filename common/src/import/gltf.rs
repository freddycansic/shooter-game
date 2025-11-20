use std::{fmt, path::PathBuf, sync::Arc};

use glium::{Display, glutin::surface::WindowSurface};
use uuid::Uuid;

use crate::geometry::Model;
use crate::geometry::Primitive;
use crate::ui;

#[derive(Debug, Clone)]
pub enum ModelLoadError {
    ModelDoesNotExist(PathBuf),
    CreateBufferError(PathBuf),
    NoPositions(PathBuf),
    NoIndices(PathBuf),
}

impl std::error::Error for ModelLoadError {}

impl fmt::Display for ModelLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ModelDoesNotExist(path) => {
                write!(f, "The model \"{:?}\" does not exist", path)
            }
            Self::CreateBufferError(path) => {
                write!(f, "Could not create buffers for the model \"{:?}\"", path)
            }
            Self::NoPositions(path) => {
                write!(
                    f,
                    "Could not extract primitive vertex positions for the model {:?}",
                    path
                )
            }
            Self::NoIndices(path) => {
                write!(
                    f,
                    "Could not extract primitive indices for the model {:?}",
                    path
                )
            }
        }
    }
}

pub fn load(
    path: PathBuf,
    display: &Display<WindowSurface>,
) -> Result<Vec<Arc<Model>>, ModelLoadError> {
    log::info!("Loading gltf {:?}...", path);

    let (document, file_buffers, _images) =
        gltf::import(&path).map_err(|_| ModelLoadError::ModelDoesNotExist(path.clone()))?;

    let models = document
        .meshes()
        .enumerate()
        .map(|(mesh_index, mesh)| {
            let primitives = mesh
                .primitives()
                .enumerate()
                .map(|(primitive_index, primitive)| {
                    log::debug!("Loading mesh {} primitive {}", mesh_index, primitive_index);

                    Primitive::from_gltf_primitive(primitive, &file_buffers, display, path.clone())
                })
                .collect::<Result<Vec<Primitive>, ModelLoadError>>()?;

            Ok(Arc::new(Model {
                uuid: Uuid::new_v4(),
                name: mesh
                    .name()
                    .unwrap_or(ui::default_name::model().as_str())
                    .to_owned(),
                primitives,
            }))
        })
        .collect::<Result<Vec<Arc<Model>>, ModelLoadError>>()?;

    Ok(models)
}
