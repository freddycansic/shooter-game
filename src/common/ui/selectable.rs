pub use proc_macros::Selectable;

pub trait Selectable {
    fn selected(&self) -> bool;

    fn select(&mut self);
    fn deselect(&mut self);
    fn toggle_selected(&mut self);
}
