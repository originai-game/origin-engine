//! og-ui's data below its stages: rectangles and insets in CSS pixels,
//! the 2D transforms and the primitives of its saved bytes, which og-ui
//! and og-rive share.

pub mod codec;
mod rect;
mod transform;

pub use rect::*;
pub use transform::*;
