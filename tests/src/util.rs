use common::engine::assets::{Assets, GeometryHandle};
use common::geometry::Geometry;
use std::path::Path;
use common::ecs::system::IntoSystem;
use common::engine::scheduler::Scheduler;
use common::runtime::ApplicationAccess;
use common::world::World;

pub fn load_test_geometry_handle(path: &Path, resources: &mut Assets) -> GeometryHandle {
    resources.get_geometry_handles(path, None).unwrap()[0]
}

pub fn load_test_geometry(path: &Path) -> Geometry {
    Geometry::load(path, None).unwrap().into_iter().next().unwrap()
}

pub struct DummyContext {
    pub commands_executed: u32,
}

impl Default for DummyContext {
    fn default() -> Self {
        Self { commands_executed: 0 }
    }
}

impl ApplicationAccess for DummyContext {
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

pub trait RunNow {
    fn run_now<S, P>(&self, system: S, world: &mut World)
    where
        S: IntoSystem<P>;
}

impl RunNow for Scheduler {
    fn run_now<S, P>(&self, system: S, world: &mut World)
    where
        S: IntoSystem<P>
    {
        system.into_system().run(world, &mut DummyContext::default());
    }
}