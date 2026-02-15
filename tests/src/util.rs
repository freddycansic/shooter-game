use common::geometry::Geometry;
use std::path::{Path, PathBuf};
use common::resources::{GeometryHandle, Resources};

pub fn load_test_geometry_handle(path: &Path, resources: &mut Resources) -> GeometryHandle {
    resources.get_geometry_handles(path, None).unwrap()[0]
}

pub fn load_test_geometry(path: &Path) -> Geometry {
    Geometry::load(path, None).unwrap().into_iter().next().unwrap()
}
