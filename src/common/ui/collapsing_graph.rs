use egui_glium::egui_winit::egui::{self, Ui};
use itertools::Itertools;
use petgraph::{
    graph::NodeIndex,
    prelude::StableDiGraph,
    visit::{Bfs, IntoNodeReferences},
    Direction,
};

use super::ui_item::UiItem;

pub fn collapsing_graph<T: UiItem>(ui: &mut Ui, graph: &mut StableDiGraph<T, ()>) {
    let top_level_nodes = graph
        .node_references()
        .filter(|(node_index, _)| {
            graph
                .neighbors_directed(*node_index, Direction::Incoming)
                .count()
                == 0
        })
        .map(|(node_index, _)| node_index)
        .collect_vec();

    for (i, node) in top_level_nodes.iter().enumerate() {
        let mut bfs = Bfs::new(&*graph, *node);

        ui.push_id(i, |ui| {
            if let Some(next) = bfs.next(&*graph) {
                collapsing_graph_inner(ui, graph, next);
            }
        });
    }
}

fn collapsing_graph_inner<T>(ui: &mut Ui, graph: &mut StableDiGraph<T, ()>, node_index: NodeIndex)
where
    T: UiItem,
{
    let model_name = graph[node_index].name();
    let children = graph
        .neighbors_directed(node_index, Direction::Outgoing)
        .collect_vec();
    let id = ui.make_persistent_id(node_index);

    if children.is_empty() {
        ui.indent(id, |ui| {
            if ui.selectable_label(false, model_name).clicked() {
                graph[node_index].toggle_selected();
            }
        });
    } else {
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                if ui.selectable_label(false, model_name).clicked() {
                    graph[node_index].toggle_selected();
                }
            })
            .body(|ui| {
                for child in children.into_iter() {
                    collapsing_graph_inner(ui, graph, child);
                }
            });
    }
}
