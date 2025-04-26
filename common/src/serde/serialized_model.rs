use itertools::Itertools;

use crate::models::Model;

use super::SerializedPrimitive;

pub struct SerializedModel {
    pub name: String,
    pub primitives: Vec<SerializedPrimitive>,
}

impl From<Model> for SerializedModel {
    fn from(value: Model) -> Self {
        Self {
            name: value.name,
            primitives: value
                .primitives
                .into_iter()
                .map(SerializedPrimitive::from)
                .collect_vec(),
        }
    }
}
