//! CSS as documents and style packs write it (docs/ui.md §3): the value
//! grammar and its conversions ([`values`]), custom properties
//! ([`variables`]), the at-rules and queries, and each specification's
//! properties lowered onto element props.
//!
//! What it takes from the document model is the one `use crate::doc`
//! below and nothing else (`crate::structure` holds that): the reflected
//! element props it lowers onto (`schema`), the name rule markup and CSS
//! share (`syntax`), the cascade's answer to whether a property is one
//! documents take (`build`), and the view-transition props, whose
//! lowering needs the document (`view_transition`). Selectors, matching
//! and the stylesheet reader stay in `doc`: they read the document tree.

pub mod at_rules;
pub(crate) mod boxes;
#[doc(hidden)]
pub mod contain_css;
pub mod container;
pub mod generated;
pub(crate) mod image_css;
pub(crate) mod interact_css;
pub mod media;
pub(crate) mod memo;
#[doc(hidden)]
pub mod motion;
pub(crate) mod multicol_css;
#[doc(hidden)]
pub mod paint_css;
pub(crate) mod position_css;
pub mod request;
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
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct StyleDeclaration {
    pub name: String,
    pub value: variables::Value,
    pub source: String,
    pub important: bool,
    /// This longhand's value must be extracted after var() substitution.
    pub pending_shorthand: Option<&'static str>,
}

/// Only these two shorthands defer extraction in `motion::expand`.
/// Restore an interned identity, never leak a snapshot's string.
impl<'de> serde::Deserialize<'de> for StyleDeclaration {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Saved {
            name: String,
            value: variables::Value,
            source: String,
            important: bool,
            pending_shorthand: Option<String>,
        }
        let saved = Saved::deserialize(deserializer)?;
        let pending_shorthand = match saved.pending_shorthand.as_deref() {
            None => None,
            Some("transition") => Some("transition"),
            Some("animation") => Some("animation"),
            Some(name) => return Err(serde::de::Error::custom(format!(
                "unknown pending shorthand {name}"
            ))),
        };
        Ok(Self { name: saved.name, value: saved.value, source: saved.source,
            important: saved.important, pending_shorthand })
    }
}

/// The enclosing stylesheet query conditions, intersected when nested.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtQuery {
    pub media: media::Query,
    /// Enclosing `@container` conditions; each picks its own container.
    pub container: Vec<container::Condition>,
}
