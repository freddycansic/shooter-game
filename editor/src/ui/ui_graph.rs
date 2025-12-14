use common::scene::graph::SceneGraph;
use egui_glium::egui_winit::egui::Ui;
use egui_ltreeview::{Action, TreeView, TreeViewBuilder};
use itertools::Itertools;
use petgraph::{Direction, graph::NodeIndex};

use crate::ui::Show;

impl Show for SceneGraph {
    fn show(&mut self, ui: &mut Ui) {
        let id = ui.make_persistent_id("Scene graph tree view");
        let (_, actions) = TreeView::new(id).show(ui, |builder| {
            let top_level_children = self
                .graph
                .neighbors_directed(self.root, Direction::Outgoing)
                .collect_vec();

            for top_level in top_level_children {
                show_tree_view_inner(self, top_level, builder);
            }
        });

        // TODO now need to make it so that when i click in the viewer without clicking an object it deselects all

        for action in actions {
            match action {
                Action::SetSelected(nodes) => {
                    for node in self.graph.node_weights_mut() {
                        node.selected = false;
                    }

                    for selected_node in nodes {
                        self.graph[NodeIndex::new(selected_node as usize)].selected = true;
                    }
                }
                _ => (),
            }
        }
    }
}

fn show_tree_view_inner(graph: &SceneGraph, node: NodeIndex, builder: &mut TreeViewBuilder<'_, i32>) {
    let children = graph.graph.neighbors_directed(node, Direction::Outgoing).collect_vec();

    if children.is_empty() {
        builder.leaf(node.index() as i32, "Leaf");
    } else {
        let is_open = builder.dir(node.index() as i32, "Dir");

        if is_open {
            for child in children {
                show_tree_view_inner(graph, child, builder);
            }
        }

        builder.close_dir();
    }
}
