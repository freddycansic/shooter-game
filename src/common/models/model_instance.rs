use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::models::{Material, Model};
use crate::transform::Transform;
use crate::ui::selectable::Selectable;

#[derive(Serialize, Deserialize, Clone, Selectable)]
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
            name: "Model".to_owned(),
            material: None,
            transform: Transform::default(),
            selected: false,
        }
    }
}
