use crate::ecs::system::{IntoSystem, System};
use crate::executor::CommandExecutor;
use crate::world::World;
use common::ecs::component::StableId;
use common::ecs::event::Event;
use fxhash::{FxHashMap, FxHashSet};

pub struct Scheduler {
    pub continuous_systems: Vec<System>,
    pub triggerable_systems: Vec<System>,
    pub triggers: FxHashMap<StableId, Vec<StableId>>,
}

impl Scheduler {
    /// These systems run continuously, once per frame.
    pub fn register_continuous<S, P>(&mut self, system: S)
    where
        S: IntoSystem<P>,
    {
        let new_system = system.into_system();

        assert!(
            !self.continuous_systems.contains(&new_system),
            "The system has already been registered"
        );

        self.continuous_systems.push(new_system);
    }

    /// These systems run once when triggered, regardless of the number of times they are triggered.
    pub fn register_triggered<E, S, P>(&mut self, system: S)
    where
        S: IntoSystem<P>,
        E: Event,
    {
        let new_system = system.into_system();

        self.triggers.entry(E::ID).or_default().push(new_system.id.clone());

        self.triggerable_systems.push(new_system);
    }

    pub fn run_systems(&mut self, world: &mut World, executor: &mut dyn CommandExecutor) {
        let mut triggered_systems_to_run = FxHashSet::<StableId>::default();

        for (event_id, system_ids) in self.triggers.iter_mut() {
            let event_queue_len = world.event_queue_from_id(event_id.clone()).0.len();

            for system_id in system_ids {
                let system = self
                    .triggerable_systems
                    .iter_mut()
                    .find(|system| system.id == *system_id)
                    .unwrap();

                let should_trigger =
                    *system.state.trigger_cursors.entry(event_id.clone()).or_insert(0) < event_queue_len;

                if should_trigger {
                    triggered_systems_to_run.insert(system_id.clone());
                    *system.state.trigger_cursors.get_mut(event_id).unwrap() = event_queue_len;
                }
            }
        }

        for system_id in triggered_systems_to_run {
            let system = self
                .triggerable_systems
                .iter_mut()
                .find(|system| system.id == system_id)
                .unwrap();

            system.run(world, executor);
        }

        for system in self.continuous_systems.iter_mut() {
            system.run(world, executor);
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            continuous_systems: vec![],
            triggerable_systems: vec![],
            triggers: FxHashMap::default(),
        }
    }
}
