//! Every prop an element takes, in the catalogue's order.
use std::sync::LazyLock;

use super::PropSpec;

pub mod css;
pub mod elements;
pub mod svg;
pub mod world;

/// Every prop: the box's and each HTML-shaped kind's, the world's, SVG's, then
/// CSS past the box.
pub static PROPS: LazyLock<Vec<PropSpec>> =
    LazyLock::new(|| [elements::CORE_PROPS, world::PROPS, svg::PROPS, css::PROPS].concat());
