use common::engine::scheduler::Scheduler;

pub struct Engine {
    pub scheduler: Scheduler,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler::default(),
        }
    }
}
