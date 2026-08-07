use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct StableId(pub u64);

impl StableId {
    pub const fn from_str(string: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_64(string.as_bytes(), None))
    }
}
