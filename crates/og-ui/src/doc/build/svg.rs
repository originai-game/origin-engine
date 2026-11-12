//! Inline SVG in documents (`docs/ui.md` §3.3): `<svg>`, `<g>` and the
//! basic shapes as retained elements, their attributes bindable.
//!
//! Each shape is one element whose draw is its path in the root `<svg>`'s
//! user space ([`crate::paint::ElementDraw::view_box`]): its fill is the
//! element's shape, drawn analytically by og-render's UI pass, and each of
//! its subpaths is a stroked outline. The view box is fitted into the box
//! when the frame is painted, so a box that resizes repaints only its
//! shapes and nothing is built again.
//!
//! SVG's presentation attributes inherit as CSS's inherited properties do:
//! an `<svg>` or `<g>` writes its effective values (and its accumulated
//! transform) into its computed map under names no stylesheet can write,
//! and each SVG element under it records reading them, so a group's bound
//! `fill` re-resolves that group and the shapes that read it, nothing
//! else.

pub use og_ui_core::doc::build::svg::*;

pub(crate) mod elements;
#[cfg(test)]
mod more_tests;
#[cfg(test)]
mod tests;

use super::*;
use crate::doc::extern_element::PRESENTATION;
use crate::doc::schema_extra::svg_prop;
use crate::paint::{ColorRef, ElementDraw, Paint, ShapeKind};
use crate::svg::{Affine, Style, bounds, shape_path};
use og_core::glam::Vec2;
