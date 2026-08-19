// OnceCell::get_mut_or_init is unstable, used for archetype columns
#![feature(once_cell_get_mut)]

// This allows this crate to be used under the name "common::" as well as "crate::"
// This means that proc-macro code which uses "common::" is valid in this crate as well as consumers
extern crate self as common;

pub mod camera;
pub mod collision;
pub mod colors;
pub mod debug;
pub mod ecs;
pub mod engine;
pub mod geometry;
pub mod gui;
pub mod import;
pub mod light;
pub mod line;
pub mod material;
pub mod maths;
pub mod quad;
pub mod runtime;
pub mod serde;
pub mod subsystems;
pub mod terrain;
pub mod world;
