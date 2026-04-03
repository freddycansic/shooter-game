use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Serialize, Deserialize)]
pub struct StableId(pub [u8; 20]);

impl StableId {
    pub const fn from_str(string: &str) -> Self {
        Self(const_sha1::sha1(string.as_bytes()).as_bytes())
    }
}