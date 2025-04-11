use std::sync::Arc;

use egui_glium::egui_winit::egui::WidgetText;
use serde::{Deserialize, Serialize};

use crate::models::{Material, Model};
use crate::transform::Transform;
use crate::ui;
use crate::ui::ui_item::UiItem;

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelInstance {
    pub model: Arc<Model>,
    pub name: String,
    pub transform: Transform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
    #[serde(skip)]
    pub selected: bool,
}

impl From<Arc<Model>> for ModelInstance {
    fn from(model: Arc<Model>) -> Self {
        Self {
            model,
            name: ui::default_name::model(),
            material: None,
            transform: Transform::default(),
            selected: false,
        }
    }
}

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
