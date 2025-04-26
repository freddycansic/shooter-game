use std::sync::Arc;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::models::Model;

use super::SerializedPrimitive;

#[derive(Serialize, Deserialize)]
pub struct SerializedModel {
    pub name: String,
    pub primitives: Vec<SerializedPrimitive>,
}

impl From<Arc<Model>> for SerializedModel {
    fn from(value: Arc<Model>) -> Self {
        Self {
            name: value.name.clone(),
            primitives: value
                .primitives
                .iter()
                .map(SerializedPrimitive::from)
                .collect_vec(),
        }
    }
}
