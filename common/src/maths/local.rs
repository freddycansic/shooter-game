use std::ops::{Deref, DerefMut};

/// Represents a type in local coordinate space. E.g. a local space ray
pub struct Local<T>(pub T);

impl<T> Deref for Local<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Local<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
