use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{models::ModelInstance, transform::Transform};

#[derive(Serialize, Deserialize)]
pub struct SerializedModelInstance {
    pub model: Uuid,
    pub name: String,
    pub transform: Transform,
    pub material: Option<Uuid>,
}

impl From<ModelInstance> for SerializedModelInstance {
    fn from(value: ModelInstance) -> Self {
        Self {
            model: value.model.uuid,
            name: value.name,
            transform: value.transform,
            material: value.material.map(|material| material.uuid),
        }
    }
}
