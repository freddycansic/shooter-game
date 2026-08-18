use crate::ecs::system::{IntoSystem, System};
use crate::runtime::ApplicationAccess;
use crate::world::World;
use common::ecs::component::StableId;
use common::ecs::events::Event;
use fxhash::{FxHashMap, FxHashSet};

#[repr(u8)]
pub enum Stage {
    Execute = 0, // On next frame, do stuff
    Pre = 1,     //
    Main = 2,    // Main game logic
    Post = 3,
    Render = 4, // After all processing is done, render
    Count,
}

pub struct SchedulerStage {
    pub continuous_systems: Vec<System>,
    pub triggerable_systems: Vec<System>,
    pub triggers: FxHashMap<StableId, Vec<StableId>>,
}

impl Default for SchedulerStage {
    fn default() -> Self {
        Self {
            continuous_systems: vec![],
            triggerable_systems: vec![],
            triggers: FxHashMap::default(),
        }
    }
}

impl SchedulerStage {
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

    fn find_triggered_systems(&mut self, world: &mut World) -> FxHashSet<StableId> {
        let mut triggered_systems = FxHashSet::<StableId>::default();

        for (event_id, system_ids) in self.triggers.iter_mut() {
            let event_queue_len = world.events_from_id(event_id.clone()).queue.len();

            for system_id in system_ids {
                let system = self
                    .triggerable_systems
                    .iter_mut()
                    .find(|system| system.id == *system_id)
                    .unwrap();

                let should_trigger =
                    *system.state.trigger_cursors.entry(event_id.clone()).or_insert(0) < event_queue_len;

                if should_trigger {
                    triggered_systems.insert(system_id.clone());
                    *system.state.trigger_cursors.get_mut(event_id).unwrap() = event_queue_len;
                }
            }
        }

        triggered_systems
    }

    fn run_triggered_systems(&mut self, world: &mut World, access: &mut dyn ApplicationAccess) {
        let triggered_systems = self.find_triggered_systems(world);

        for system_id_to_trigger in triggered_systems.into_iter() {
            let system = self
                .triggerable_systems
                .iter_mut()
                .find(|system| system.id == system_id_to_trigger)
                .unwrap();

            system.run(world, access);
        }
    }

    fn run_continuous_systems(&mut self, world: &mut World, access: &mut dyn ApplicationAccess) {
        for system in self.continuous_systems.iter_mut() {
            system.run(world, access);
        }
    }

    pub fn run(&mut self, world: &mut World, access: &mut dyn ApplicationAccess) {
        self.run_triggered_systems(world, access);
        self.run_continuous_systems(world, access);
    }
}

pub struct Scheduler {
    stages: [SchedulerStage; Stage::Count as usize],
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            stages: core::array::from_fn(|_| SchedulerStage::default()),
        }
    }
}

impl Scheduler {
    /// These systems run continuously, once per frame.
    pub fn register_continuous<S, P>(&mut self, system: S, stage: Stage)
    where
        S: IntoSystem<P>,
    {
        self.stages[stage as usize].register_continuous(system);
    }

    /// These systems run once when triggered, regardless of the number of times they are triggered.
    pub fn register_triggered<E, S, P>(&mut self, system: S, stage: Stage)
    where
        S: IntoSystem<P>,
        E: Event,
    {
        self.stages[stage as usize].register_triggered::<E, S, P>(system);
    }

    pub fn run(&mut self, world: &mut World, access: &mut dyn ApplicationAccess) {
        world.consume_external_events();
        
        for stage in self.stages.iter_mut() {
            stage.run(world, access);
        }
    }
}
