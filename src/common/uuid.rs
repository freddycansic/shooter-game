use std::hash::{Hash, Hasher};

use once_cell::sync::OnceCell;

static CURRENT: OnceCell<u128> = OnceCell::new();

#[derive(PartialEq, Eq)]
pub struct UUID(u128);

impl Default for UUID {
    fn default() -> Self {
        Self::new()
    }
}

impl UUID {
    pub fn new() -> Self {
        Self(CURRENT.get_or_init(|| 0_u128) + 1)
    }
}

impl Hash for UUID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
