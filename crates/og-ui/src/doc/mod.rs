//! Scriptless Svelte documents and scoped CSS, validated against [`schema`].
//! [`UiDocument::parse`] produces reflected nodes, expressions, and cascade
//! rules for the retained builder. The cook stores the validated document
//! with its base string-table dependencies in an `og.UiDocument` blob.
//! [`UiDocument::set_prop`] saves probe edits without rewriting other nodes.

pub use og_ui_core::doc::*;

pub mod build;
mod script;
pub(crate) mod svelte;
mod writeback;

use crate::bind::Expr;
use og_core::ogt::{Number, Value};
use schema::Ty;
