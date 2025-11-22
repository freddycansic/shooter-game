use std::sync::Arc;

use egui_glium::egui_winit::egui::WidgetText;

use crate::geometry::Geometry;
// use crate::geometry::{Material, Model};
use crate::transform::Transform;
use crate::ui;
use crate::ui::UiItem;

#[derive(Clone)]
pub struct ModelInstance {
    pub model: Arc<Geometry>,
    pub name: String,
    pub transform: Transform,
    // pub material: Option<Arc<Material>>,
    pub selected: bool,
}

// impl ModelInstance {
//     fn new(model: Arc<Model>, collider_set: &mut ColliderSet) -> Self {
//         Self {
//             model,
//             name: ui::default_name::model(),
//             material: None,
//             transform: Transform::default(),
//             selected: false,
//         }
//     }
// }

// impl From<Arc<Model>> for ModelInstance {
//     fn from(model: Arc<Model>) -> Self {
//         Self {
//             model,
//             name: ui::default_name::model(),
//             material: None,
//             transform: Transform::default(),
//             selected: false,
//         }
//     }
// }

impl UiItem for ModelInstance {
    fn name(&self) -> WidgetText {
        (&self.name).into()
    }

    // TODO make it proc macro if this becomes a problem
    fn selected(&self) -> bool {
        self.selected
    }

    fn select(&mut self) {
        self.selected = true;
    }

    fn deselect(&mut self) {
        self.selected = false;
    }
}
