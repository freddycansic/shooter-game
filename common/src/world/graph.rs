// #[derive(Clone, Serialize, Deserialize)]
// pub struct WorldNode {
//     pub local_transform: Transform,
//     pub visible: bool,
//
//     #[serde(skip)]
//     world_transform: Transform,
//     #[serde(skip)]
//     pub world_transform_dirty: bool,
// }

// pub fn calculate_world_matrices(&mut self) {
//     self.calculate_world_matrices_inner(self.root);
// }
//
// fn calculate_world_matrices_inner(&mut self, parent: NodeIndex) {
//     let children = self.graph.neighbors_directed(parent, Direction::Outgoing).collect_vec();
//
//     for child in children.into_iter() {
//         self.graph[child].world_transform = self.graph[parent]
//             .world_transform
//             .combine(&self.graph[child].local_transform);
//
//         self.graph[child].world_transform_dirty = false;
//
//         self.calculate_world_matrices_inner(child);
//     }
// }
