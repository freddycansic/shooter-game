use common::engine::assets::{Assets, GeometryHandle};
use common::executor::CommandExecutor;
use common::geometry::Geometry;
use std::path::Path;

pub fn load_test_geometry_handle(path: &Path, resources: &mut Assets) -> GeometryHandle {
    resources.get_geometry_handles(path, None).unwrap()[0]
}

pub fn load_test_geometry(path: &Path) -> Geometry {
    Geometry::load(path, None).unwrap().into_iter().next().unwrap()
}

pub struct DummyExecutor {
    pub commands_executed: u32,
}

impl Default for DummyExecutor {
    fn default() -> Self {
        Self { commands_executed: 0 }
    }
}

impl CommandExecutor for DummyExecutor {
    fn capture_cursor(&mut self) {
        self.commands_executed += 1;
    }
    fn release_cursor(&mut self) {
        self.commands_executed += 1;
    }
    fn center_cursor(&mut self) {
        self.commands_executed += 1;
    }
    fn exit(&mut self) {
        self.commands_executed += 1;
    }
}
