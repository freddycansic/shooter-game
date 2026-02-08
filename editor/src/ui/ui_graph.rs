use common::world::WorldGraph;
use egui_glium::egui_winit::egui::Ui;
use egui_ltreeview::{Action, TreeView, TreeViewBuilder};
use itertools::Itertools;
use petgraph::{Direction, graph::NodeIndex};

use crate::ui::Show;

impl Show for WorldGraph {
    fn show(&mut self, ui: &mut Ui) {
        let id = ui.make_persistent_id("World graph tree view");
        let (_, actions) = TreeView::new(id).show(ui, |builder| {
            let top_level_children = self
                .graph
                .neighbors_directed(self.root, Direction::Outgoing)
                .collect_vec();

            for top_level in top_level_children {
                show_tree_view_inner(self, top_level, builder);
            }
        });

        for action in actions {
            match action {
                Action::SetSelected(nodes) => {
                    self.selection = nodes
                        .into_iter()
                        .map(|index| NodeIndex::new(index as usize))
                        .collect_vec();
                }
                _ => (),
            }
        }
    }
}

fn show_tree_view_inner(graph: &WorldGraph, node: NodeIndex, builder: &mut TreeViewBuilder<'_, i32>) {
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
