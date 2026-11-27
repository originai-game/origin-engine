//! Native 2D skeletal animation: parent-first bones, slots in draw order,
//! regions, weighted meshes, boxes, paths, child armatures, constraints,
//! animation channels and skins. Author declarations resolve names in the
//! cook; the runtime reads validated native fields, not an author language.
//!
//! Coordinates are pixels with +y up, degrees counter-clockwise; `unit`
//! pixels make one world unit. Bone keys offset the setup pose: rotation,
//! skew and translation add, scale multiplies. Skin swaps keep the same slot.
//! A child inherits the parent's clip and skin by name, falling back to its
//! own chosen clip and default skin. Transform and path constraints follow IK.
//!
//! Every angle uses [`sincos`] and [`atan2`] with f64 arithmetic and square
//! roots; Bézier easing uses bisection, so a pose has the same bits on hosts.
mod asset;
mod constraints;
mod cooked;
mod math;
mod types;
mod validate;

use constraints::{fabrik, path_constraint, sq, transform_constraint, world_of};
pub use math::{Affine2, atan2, bezier, check_curve, sincos};
use math::{DEG, wrap};
use types::Body;
pub use types::*;

use crate::SheetFilter;
use og_core::{Error, ErrorKind, Result};
