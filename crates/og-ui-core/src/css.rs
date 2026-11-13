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

use crate::doc::{build, schema, syntax, view_transition};
use og_core::ogt::Value;

/// One authored CSS declaration. Values remain token streams until the
/// winning declaration's variables are resolved against its mounted parent.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleDeclaration {
    pub name: String,
    pub value: variables::Value,
    pub source: String,
    pub important: bool,
    /// This longhand's value must be extracted after var() substitution.
    pub pending_shorthand: Option<&'static str>,
}

/// The enclosing stylesheet query conditions, intersected when nested.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtQuery {
    pub media: media::Query,
    /// Enclosing `@container` conditions; each picks its own container.
    pub container: Vec<container::Condition>,
}
