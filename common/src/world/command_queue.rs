use fxhash::FxHashMap;
use sorted_vec::ReverseSortedVec;
use common::ecs::entity::Entity;
use common::ecs::owned_components::OwnedComponents;

pub struct CommandQueue {
    pub spawn_queue: Vec<Box<dyn OwnedComponents>>,
    pub destroy_queue: FxHashMap<u64, ReverseSortedVec<u32>>, // grouped by archetype, sorted in reverse order of entity rows indices
                                                              // so we can delete in order
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self {
            spawn_queue: vec![],
            destroy_queue: FxHashMap::default(),
        }
    }
}

impl CommandQueue {
    pub fn queue_spawn<T: OwnedComponents + 'static>(&mut self, components: T) {
        self.spawn_queue.push(Box::new(components));
    }
    
    pub fn queue_destroy(&mut self, entity: Entity) {
        self.destroy_queue.entry(entity.archetype_id).or_default().insert(std::cmp::Reverse(entity.row));
    }
}