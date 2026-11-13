//! CSS as documents and style packs write it (docs/ui.md §3): the value
//! grammar and its conversions ([`values`]), custom properties
//! ([`variables`]), the at-rules and queries, and each specification's
//! properties lowered onto element props.
//!
//! What it takes from the document model is named here and nowhere else
//! in the module: the reflected element props it lowers onto and the
//! document's value types.

pub mod at_rules;
pub(crate) mod boxes;
#[doc(hidden)]
pub mod contain_css;
pub mod container;
pub mod generated;
pub(crate) mod image_css;
pub(crate) mod interact_css;
pub mod media;
#[doc(hidden)]
pub mod motion;
pub(crate) mod multicol_css;
#[doc(hidden)]
pub mod paint_css;
pub(crate) mod position_css;
pub(crate) mod scroll_css;
pub(crate) mod text_css;
pub(crate) mod tracks;
pub mod values;
pub mod variables;

pub(crate) use og_css::calc;
use values as css;

use crate::doc::{AtQuery, StyleDeclaration, build, lists, syntax, view_transition};
use og_core::ogt::Value;
